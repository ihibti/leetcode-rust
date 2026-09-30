use crate::catalog::{self, CatalogError, Difficulty, Problem, Source, Window};
use crate::leetcode::FetchError;
use crate::remote::{Remote, RemoteError};
use std::collections::HashSet;
use std::fmt;
use std::io;
use std::path::Path;

const SOURCE: Source = Source::Liquidslr;
const SOURCE_BROWSE_URL: &str = "https://github.com/liquidslr/leetcode-company-wise-problems";

pub struct NextArgs {
    pub window: Window,
    pub company: String,
    pub difficulty: Option<Difficulty>,
    pub force: bool,
}

#[derive(Debug, PartialEq)]
pub enum SkipReason {
    Premium,
    NoRust,
}

#[derive(Debug)]
pub struct Skip {
    pub title: String,
    pub reason: SkipReason,
}

#[derive(Debug, PartialEq)]
pub enum Exhaustion {
    AllDone { total: usize },
    FilterEmpty { difficulty: Difficulty, done: usize },
    AllUnavailable { skipped: usize },
}

#[derive(Debug)]
pub enum Outcome {
    Picked(Problem),
    Exhausted(Exhaustion),
}

#[derive(Debug)]
pub struct Report {
    pub outcome: Outcome,
    pub skipped: Vec<Skip>,
    pub malformed_rows: usize,
}

#[derive(Debug)]
pub enum NextError {
    Unsaved(io::Error),
    CompanyNotFound(String),
    Catalog(CatalogError),
    Network(String),
    Fetch(FetchError),
    Io(io::Error),
}

impl fmt::Display for NextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NextError::Unsaved(e) | NextError::Io(e) => write!(f, "{e}"),
            NextError::CompanyNotFound(company) => write!(
                f,
                "no problem list for company \"{company}\" (names are case-sensitive, e.g. Amazon, Google, Meta)\n  browse: {SOURCE_BROWSE_URL}"
            ),
            NextError::Catalog(e) => write!(f, "{e}"),
            NextError::Network(msg) => write!(f, "could not download the problem list: {msg}"),
            NextError::Fetch(e) => write!(f, "{e}"),
        }
    }
}

impl fmt::Display for SkipReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkipReason::Premium => write!(f, "premium"),
            SkipReason::NoRust => write!(f, "no Rust"),
        }
    }
}

pub fn candidates<'a>(
    problems: &'a [Problem],
    done: &HashSet<String>,
    difficulty: Option<Difficulty>,
) -> Vec<&'a Problem> {
    problems
        .iter()
        .filter(|p| !done.contains(&p.slug))
        .filter(|p| difficulty.is_none_or(|d| p.difficulty == d))
        .collect()
}

pub fn weight(problem: &Problem) -> f64 {
    problem.frequency.max(1.0)
}

pub fn pick_weighted<'a>(pool: &[&'a Problem], roll: f64) -> Option<&'a Problem> {
    let total: f64 = pool.iter().map(|p| weight(p)).sum();
    let target = roll.clamp(0.0, 1.0) * total;
    let mut cumulative = 0.0;
    for &problem in pool {
        cumulative += weight(problem);
        if target < cumulative {
            return Some(problem);
        }
    }
    pool.last().copied()
}

pub fn exhaustion(
    problems: &[Problem],
    done: &HashSet<String>,
    difficulty: Option<Difficulty>,
) -> Exhaustion {
    let remaining = problems.iter().filter(|p| !done.contains(&p.slug)).count();
    match difficulty {
        Some(d) if remaining > 0 => Exhaustion::FilterEmpty {
            difficulty: d,
            done: problems
                .iter()
                .filter(|p| p.difficulty == d && done.contains(&p.slug))
                .count(),
        },
        _ => Exhaustion::AllDone {
            total: problems.len(),
        },
    }
}

pub fn exhaustion_message(exhaustion: &Exhaustion, company: &str, window: Window) -> String {
    match exhaustion {
        Exhaustion::AllDone { total: 0 } => {
            format!("{company} {window}: the list is empty")
        }
        Exhaustion::AllDone { total } => format!(
            "{company} {window}: all {total} done{}",
            widen_hint(window).map(|w| format!(" — try `cargo next {w}`")).unwrap_or_default()
        ),
        Exhaustion::FilterEmpty { difficulty, done } => format!(
            "{company} {window}: no {difficulty} problems left ({done} done) — drop -d or widen the window"
        ),
        Exhaustion::AllUnavailable { skipped } => format!(
            "{company} {window}: the {skipped} remaining problems are premium-only or have no Rust template"
        ),
    }
}

fn widen_hint(window: Window) -> Option<Window> {
    match window {
        Window::Days30 => Some(Window::Months3),
        Window::Months3 => Some(Window::Months6),
        Window::Months6 => Some(Window::All),
        Window::Older | Window::All => None,
    }
}

pub fn run(
    root: &Path,
    args: &NextArgs,
    remote: &impl Remote,
    mut rolls: impl FnMut() -> f64,
) -> Result<Report, NextError> {
    crate::solve::ensure_free(root, args.force).map_err(NextError::Unsaved)?;
    let done = crate::progress::done_slugs(&root.join("archive")).map_err(NextError::Io)?;

    let url = catalog::csv_url(SOURCE, &args.company, args.window);
    let text = remote.get_text(&url).map_err(|e| match e {
        RemoteError::NotFound => NextError::CompanyNotFound(args.company.clone()),
        RemoteError::Other(msg) => NextError::Network(msg),
    })?;
    let catalog = catalog::parse_csv(SOURCE, &text).map_err(NextError::Catalog)?;

    let mut report = Report {
        outcome: Outcome::Exhausted(Exhaustion::AllDone { total: 0 }),
        skipped: Vec::new(),
        malformed_rows: catalog.skipped_rows,
    };

    let mut pool = candidates(&catalog.problems, &done, args.difficulty);
    if pool.is_empty() {
        report.outcome =
            Outcome::Exhausted(exhaustion(&catalog.problems, &done, args.difficulty));
        return Ok(report);
    }

    while let Some(problem) = pick_weighted(&pool, rolls()) {
        let reason = match remote.problem(&problem.slug) {
            Ok(data) => {
                crate::solve::start(root, Some(&data)).map_err(NextError::Io)?;
                report.outcome = Outcome::Picked(problem.clone());
                return Ok(report);
            }
            Err(FetchError::PaidOnly) => SkipReason::Premium,
            Err(FetchError::NoRustSnippet) => SkipReason::NoRust,
            Err(e) => return Err(NextError::Fetch(e)),
        };
        report.skipped.push(Skip {
            title: problem.title.clone(),
            reason,
        });
        pool.retain(|p| p.slug != problem.slug);
    }

    report.outcome = Outcome::Exhausted(Exhaustion::AllUnavailable {
        skipped: report.skipped.len(),
    });
    Ok(report)
}

pub fn cli(root: &Path, args: &NextArgs) -> Result<(), io::Error> {
    eprintln!("Drawing from {} {}...", args.company, args.window);
    let report = run(root, args, &crate::remote::UreqRemote, rand::random::<f64>)
        .map_err(|e| io::Error::other(e.to_string()))?;

    if report.malformed_rows > 0 {
        eprintln!(
            "Warning: ignored {} malformed rows in the {} {} list",
            report.malformed_rows, args.company, args.window
        );
    }
    for skip in &report.skipped {
        eprintln!("skip  {} ({})", skip.title, skip.reason);
    }

    match report.outcome {
        Outcome::Picked(p) => {
            println!(
                "{} · freq {} · {}\n  https://leetcode.com/problems/{}/",
                p.difficulty, p.frequency, p.title, p.slug
            );
            println!("Ready! Open src/solution.rs and start coding");
        }
        Outcome::Exhausted(e) => println!("{}", exhaustion_message(&e, &args.company, args.window)),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::leetcode::ProblemData;
    use crate::solve::SOLUTION_TEMPLATE;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    const CSV: &str = "\
Difficulty,Title,Frequency,Acceptance Rate,Link,Topics
EASY,Two Sum,100.0,0.1,https://leetcode.com/problems/two-sum,\"Array, Hash Table\"
MEDIUM,Merge Intervals,75.8,0.1,https://leetcode.com/problems/merge-intervals,Array
";

    #[derive(Clone, Copy)]
    enum Answer {
        Ok,
        PaidOnly,
        NoRust,
        Network,
    }

    struct FakeRemote {
        csv: Option<&'static str>,
        answers: HashMap<&'static str, Answer>,
        calls: RefCell<Vec<String>>,
    }

    impl FakeRemote {
        fn new(answers: &[(&'static str, Answer)]) -> Self {
            FakeRemote {
                csv: Some(CSV),
                answers: answers.iter().copied().collect(),
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl Remote for FakeRemote {
        fn get_text(&self, url: &str) -> Result<String, RemoteError> {
            self.calls.borrow_mut().push(url.to_string());
            self.csv.map(str::to_string).ok_or(RemoteError::NotFound)
        }

        fn problem(&self, slug: &str) -> Result<ProblemData, FetchError> {
            self.calls.borrow_mut().push(slug.to_string());
            match self.answers.get(slug).copied().unwrap_or(Answer::Ok) {
                Answer::Ok => Ok(ProblemData {
                    slug: slug.to_string(),
                    title: slug.to_string(),
                    difficulty: "Easy".into(),
                    tags: vec![],
                    examples_text: String::new(),
                    rust_snippet: format!(
                        "impl Solution {{\n    pub fn {}() {{}}\n}}",
                        slug.replace('-', "_")
                    ),
                }),
                Answer::PaidOnly => Err(FetchError::PaidOnly),
                Answer::NoRust => Err(FetchError::NoRustSnippet),
                Answer::Network => Err(FetchError::Network("offline".into())),
            }
        }
    }

    fn workspace() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/solution.rs"), SOLUTION_TEMPLATE).unwrap();
        dir
    }

    fn archive_slug(dir: &TempDir, slug: &str) {
        fs::create_dir_all(dir.path().join("archive")).unwrap();
        fs::write(
            dir.path().join(format!("archive/{slug}.rs")),
            format!("//! Problem: {slug}\n//! Slug: {slug}\n\ncode\n"),
        )
        .unwrap();
    }

    fn args(difficulty: Option<Difficulty>) -> NextArgs {
        NextArgs {
            window: Window::Days30,
            company: "Amazon".into(),
            difficulty,
            force: false,
        }
    }

    fn session(dir: &TempDir) -> String {
        fs::read_to_string(dir.path().join(".solve_session")).unwrap()
    }

    fn problem(slug: &str, frequency: f64, difficulty: Difficulty) -> Problem {
        Problem {
            slug: slug.into(),
            title: slug.into(),
            difficulty,
            frequency,
            topics: vec![],
        }
    }

    #[test]
    fn candidates_excludes_done() {
        let problems = vec![
            problem("a", 50.0, Difficulty::Easy),
            problem("b", 50.0, Difficulty::Easy),
        ];
        let done = HashSet::from(["a".to_string()]);
        let pool = candidates(&problems, &done, None);
        assert_eq!(pool.len(), 1);
        assert_eq!(pool[0].slug, "b");
    }

    #[test]
    fn candidates_filters_difficulty() {
        let problems = vec![
            problem("a", 50.0, Difficulty::Easy),
            problem("b", 50.0, Difficulty::Hard),
        ];
        let pool = candidates(&problems, &HashSet::new(), Some(Difficulty::Hard));
        assert_eq!(pool.len(), 1);
        assert_eq!(pool[0].slug, "b");
    }

    #[test]
    fn weight_floor() {
        assert_eq!(weight(&problem("a", 0.0, Difficulty::Easy)), 1.0);
        assert_eq!(weight(&problem("a", 61.3, Difficulty::Easy)), 61.3);
    }

    #[test]
    fn pick_first_at_zero() {
        let problems = [
            problem("a", 10.0, Difficulty::Easy),
            problem("b", 10.0, Difficulty::Easy),
        ];
        let pool: Vec<&Problem> = problems.iter().collect();
        assert_eq!(pick_weighted(&pool, 0.0).unwrap().slug, "a");
    }

    #[test]
    fn pick_last_near_one() {
        let problems = [
            problem("a", 39.1, Difficulty::Easy),
            problem("b", 48.9, Difficulty::Easy),
            problem("c", 61.3, Difficulty::Easy),
        ];
        let pool: Vec<&Problem> = problems.iter().collect();
        assert_eq!(pick_weighted(&pool, 0.999_999_9).unwrap().slug, "c");
    }

    #[test]
    fn pick_mid_bucket() {
        let problems = [
            problem("a", 1.0, Difficulty::Easy),
            problem("b", 1.0, Difficulty::Easy),
            problem("c", 2.0, Difficulty::Easy),
        ];
        let pool: Vec<&Problem> = problems.iter().collect();
        assert_eq!(pick_weighted(&pool, 0.24).unwrap().slug, "a");
        assert_eq!(pick_weighted(&pool, 0.26).unwrap().slug, "b");
        assert_eq!(pick_weighted(&pool, 0.5).unwrap().slug, "c");
    }

    #[test]
    fn pick_empty_none() {
        assert!(pick_weighted(&[], 0.3).is_none());
    }

    #[test]
    fn run_happy_path() {
        let dir = workspace();
        let remote = FakeRemote::new(&[]);
        let report = run(dir.path(), &args(None), &remote, || 0.0).unwrap();

        assert!(matches!(report.outcome, Outcome::Picked(ref p) if p.slug == "two-sum"));
        let solution = fs::read_to_string(dir.path().join("src/solution.rs")).unwrap();
        assert!(solution.contains("pub fn two_sum()"));
        assert!(session(&dir).contains("\"slug\":\"two-sum\""));
    }

    #[test]
    fn run_rerolls_paid_only() {
        let dir = workspace();
        let remote = FakeRemote::new(&[("two-sum", Answer::PaidOnly)]);
        let report = run(dir.path(), &args(None), &remote, || 0.0).unwrap();

        assert!(matches!(report.outcome, Outcome::Picked(ref p) if p.slug == "merge-intervals"));
        assert_eq!(report.skipped.len(), 1);
        assert_eq!(report.skipped[0].reason, SkipReason::Premium);
        assert!(session(&dir).contains("\"slug\":\"merge-intervals\""));
    }

    #[test]
    fn run_network_error_leaves_workspace() {
        let dir = workspace();
        let remote = FakeRemote::new(&[("two-sum", Answer::Network)]);
        let result = run(dir.path(), &args(None), &remote, || 0.0);

        assert!(matches!(result, Err(NextError::Fetch(FetchError::Network(_)))));
        let solution = fs::read_to_string(dir.path().join("src/solution.rs")).unwrap();
        assert_eq!(solution, SOLUTION_TEMPLATE);
        assert!(!dir.path().join(".solve_session").exists());
    }

    #[test]
    fn run_all_unavailable_exhausted() {
        let dir = workspace();
        let remote = FakeRemote::new(&[
            ("two-sum", Answer::PaidOnly),
            ("merge-intervals", Answer::NoRust),
        ]);
        let report = run(dir.path(), &args(None), &remote, || 0.0).unwrap();

        assert!(matches!(
            report.outcome,
            Outcome::Exhausted(Exhaustion::AllUnavailable { skipped: 2 })
        ));
        assert!(!dir.path().join(".solve_session").exists());
    }

    #[test]
    fn run_dirty_workspace_no_calls() {
        let dir = workspace();
        fs::write(dir.path().join("src/solution.rs"), "work in progress").unwrap();
        let remote = FakeRemote::new(&[]);
        let result = run(dir.path(), &args(None), &remote, || 0.0);

        assert!(matches!(result, Err(NextError::Unsaved(_))));
        assert!(remote.calls.borrow().is_empty());
    }

    #[test]
    fn run_skips_archived() {
        for roll in [0.0, 0.5, 0.999] {
            let dir = workspace();
            archive_slug(&dir, "two-sum");
            let remote = FakeRemote::new(&[]);
            let report = run(dir.path(), &args(None), &remote, || roll).unwrap();
            assert!(matches!(report.outcome, Outcome::Picked(ref p) if p.slug == "merge-intervals"));
        }
    }

    #[test]
    fn run_all_done() {
        let dir = workspace();
        archive_slug(&dir, "two-sum");
        archive_slug(&dir, "merge-intervals");
        let remote = FakeRemote::new(&[]);
        let report = run(dir.path(), &args(Some(Difficulty::Easy)), &remote, || 0.0).unwrap();

        assert!(matches!(
            report.outcome,
            Outcome::Exhausted(Exhaustion::AllDone { total: 2 })
        ));
    }

    #[test]
    fn run_filter_empty() {
        let dir = workspace();
        archive_slug(&dir, "two-sum");
        let remote = FakeRemote::new(&[]);
        let report = run(dir.path(), &args(Some(Difficulty::Easy)), &remote, || 0.0).unwrap();

        assert!(matches!(
            report.outcome,
            Outcome::Exhausted(Exhaustion::FilterEmpty { difficulty: Difficulty::Easy, done: 1 })
        ));
    }

    #[test]
    fn run_company_not_found() {
        let dir = workspace();
        let mut remote = FakeRemote::new(&[]);
        remote.csv = None;
        let mut lower = args(None);
        lower.company = "amazon".into();
        let result = run(dir.path(), &lower, &remote, || 0.0);

        match result {
            Err(e @ NextError::CompanyNotFound(_)) => {
                assert!(e.to_string().contains("\"amazon\""));
                assert!(e.to_string().contains("case-sensitive"));
            }
            other => panic!("expected CompanyNotFound, got {other:?}"),
        }
    }

    #[test]
    fn message_all_done_suggests_wider_window() {
        let msg = exhaustion_message(&Exhaustion::AllDone { total: 171 }, "Amazon", Window::Days30);
        assert_eq!(msg, "Amazon 30d: all 171 done — try `cargo next 3m`");
    }

    #[test]
    fn message_all_done_widest_has_no_hint() {
        let msg = exhaustion_message(&Exhaustion::AllDone { total: 5 }, "Amazon", Window::All);
        assert_eq!(msg, "Amazon all: all 5 done");
    }
}
