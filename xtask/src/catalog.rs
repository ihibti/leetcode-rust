use clap::ValueEnum;
use std::collections::HashSet;
use std::fmt;

const LIQUIDSLR_BASE: &str =
    "https://raw.githubusercontent.com/liquidslr/leetcode-company-wise-problems/main";
const LIQUIDSLR_HEADER: &str = "Difficulty,Title,Frequency,Acceptance Rate,Link,Topics";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Liquidslr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Window {
    #[value(name = "30d")]
    Days30,
    #[value(name = "3m")]
    Months3,
    #[value(name = "6m")]
    Months6,
    #[value(name = "6m+")]
    Older,
    #[value(name = "all")]
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    pub slug: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub frequency: f64,
    pub topics: Vec<String>,
}

#[derive(Debug)]
pub struct Catalog {
    pub problems: Vec<Problem>,
    pub skipped_rows: usize,
}

#[derive(Debug)]
pub enum CatalogError {
    SchemaChanged { expected: String, found: String },
    Csv(String),
}

impl Difficulty {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "easy" => Some(Difficulty::Easy),
            "medium" => Some(Difficulty::Medium),
            "hard" => Some(Difficulty::Hard),
            _ => None,
        }
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        };
        write!(f, "{label}")
    }
}

impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.to_possible_value().expect("no skipped variants");
        write!(f, "{}", value.get_name())
    }
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CatalogError::SchemaChanged { expected, found } => write!(
                f,
                "problem list format changed upstream\n  expected: {expected}\n  found:    {found}"
            ),
            CatalogError::Csv(msg) => write!(f, "could not read problem list: {msg}"),
        }
    }
}

pub fn encode_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

pub fn csv_url(source: Source, company: &str, window: Window) -> String {
    match source {
        Source::Liquidslr => {
            let file = match window {
                Window::Days30 => "1. Thirty Days.csv",
                Window::Months3 => "2. Three Months.csv",
                Window::Months6 => "3. Six Months.csv",
                Window::Older => "4. More Than Six Months.csv",
                Window::All => "5. All.csv",
            };
            format!(
                "{LIQUIDSLR_BASE}/{}/{}",
                encode_segment(company),
                encode_segment(file)
            )
        }
    }
}

pub fn parse_csv(source: Source, text: &str) -> Result<Catalog, CatalogError> {
    match source {
        Source::Liquidslr => parse_liquidslr(text),
    }
}

fn parse_liquidslr(text: &str) -> Result<Catalog, CatalogError> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(text.as_bytes());

    let header = reader
        .headers()
        .map_err(|e| CatalogError::Csv(e.to_string()))?
        .iter()
        .collect::<Vec<_>>()
        .join(",");
    if header != LIQUIDSLR_HEADER {
        return Err(CatalogError::SchemaChanged {
            expected: LIQUIDSLR_HEADER.to_string(),
            found: header,
        });
    }

    let mut problems = Vec::new();
    let mut seen = HashSet::new();
    let mut skipped_rows = 0;

    for record in reader.records() {
        let Some(problem) = record.ok().and_then(|r| liquidslr_row(&r)) else {
            skipped_rows += 1;
            continue;
        };
        if seen.insert(problem.slug.clone()) {
            problems.push(problem);
        }
    }

    Ok(Catalog {
        problems,
        skipped_rows,
    })
}

fn liquidslr_row(record: &csv::StringRecord) -> Option<Problem> {
    let difficulty = Difficulty::parse(record.get(0)?)?;
    let title = record.get(1)?.trim().to_string();
    let frequency: f64 = record.get(2)?.trim().parse().ok()?;
    if !frequency.is_finite() {
        return None;
    }
    let slug = crate::leetcode::extract_slug(record.get(4)?)?;
    let topics = record
        .get(5)
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect();

    Some(Problem {
        slug,
        title,
        difficulty,
        frequency,
        topics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "\
Difficulty,Title,Frequency,Acceptance Rate,Link,Topics
EASY,Two Sum,100.0,0.005800918768980581,https://leetcode.com/problems/two-sum,\"Array, Hash Table\"
HARD,Trapping Rain Water,80.7,0.006802783224080567,https://leetcode.com/problems/trapping-rain-water,\"Array, Two Pointers, Dynamic Programming, Stack, Monotonic Stack\"
MEDIUM,\"Pow(x, n)\",39.1,0.003917051350448515,https://leetcode.com/problems/powx-n,\"Math, Recursion\"
MEDIUM,Merge Intervals,75.8,0.005257590403796301,https://leetcode.com/problems/merge-intervals,Array
";

    fn parse(text: &str) -> Catalog {
        parse_csv(Source::Liquidslr, text).unwrap()
    }

    #[test]
    fn encode_space_and_dot() {
        assert_eq!(encode_segment("1. Thirty Days.csv"), "1.%20Thirty%20Days.csv");
        assert_eq!(encode_segment("AT&T"), "AT%26T");
        assert_eq!(encode_segment("Amazon"), "Amazon");
    }

    #[test]
    fn url_each_window() {
        let base = "https://raw.githubusercontent.com/liquidslr/leetcode-company-wise-problems/main/Amazon/";
        let cases = [
            (Window::Days30, "1.%20Thirty%20Days.csv"),
            (Window::Months3, "2.%20Three%20Months.csv"),
            (Window::Months6, "3.%20Six%20Months.csv"),
            (Window::Older, "4.%20More%20Than%20Six%20Months.csv"),
            (Window::All, "5.%20All.csv"),
        ];
        for (window, file) in cases {
            assert_eq!(csv_url(Source::Liquidslr, "Amazon", window), format!("{base}{file}"));
        }
    }

    #[test]
    fn url_company_with_space() {
        assert!(csv_url(Source::Liquidslr, "Goldman Sachs", Window::All).contains("/Goldman%20Sachs/"));
    }

    #[test]
    fn parse_fixture() {
        let catalog = parse(FIXTURE);
        assert_eq!(catalog.skipped_rows, 0);
        assert_eq!(catalog.problems.len(), 4);

        let pow = &catalog.problems[2];
        assert_eq!(pow.slug, "powx-n");
        assert_eq!(pow.title, "Pow(x, n)");
        assert_eq!(pow.difficulty, Difficulty::Medium);
        assert_eq!(pow.frequency, 39.1);
        assert_eq!(pow.topics, vec!["Math", "Recursion"]);

        assert_eq!(catalog.problems[1].difficulty, Difficulty::Hard);
        assert_eq!(catalog.problems[3].topics, vec!["Array"]);
    }

    #[test]
    fn parse_crlf() {
        let crlf = FIXTURE.replace('\n', "\r\n");
        assert_eq!(parse(&crlf).problems, parse(FIXTURE).problems);
    }

    #[test]
    fn parse_schema_changed() {
        let other = "ID,URL,Title,Difficulty,Acceptance %,Frequency %\n1,https://leetcode.com/problems/two-sum,Two Sum,Easy,57.8%,100.0%\n";
        assert!(matches!(
            parse_csv(Source::Liquidslr, other),
            Err(CatalogError::SchemaChanged { .. })
        ));
    }

    #[test]
    fn parse_bad_row_skipped() {
        let text = format!("{FIXTURE}EASY,Broken,n/a,0.1,https://leetcode.com/problems/broken,Array\n");
        let catalog = parse(&text);
        assert_eq!(catalog.skipped_rows, 1);
        assert_eq!(catalog.problems.len(), 4);
    }

    #[test]
    fn parse_duplicate_link_keeps_first() {
        let text = format!("{FIXTURE}HARD,Two Sum Again,10.0,0.1,https://leetcode.com/problems/two-sum,Array\n");
        let catalog = parse(&text);
        assert_eq!(catalog.problems.len(), 4);
        assert_eq!(catalog.problems[0].title, "Two Sum");
    }

    #[test]
    fn difficulty_parse_cases() {
        assert_eq!(Difficulty::parse("EASY"), Some(Difficulty::Easy));
        assert_eq!(Difficulty::parse("easy"), Some(Difficulty::Easy));
        assert_eq!(Difficulty::parse("Easy"), Some(Difficulty::Easy));
        assert_eq!(Difficulty::parse("x"), None);
    }

    #[test]
    fn window_display_matches_cli() {
        assert_eq!(Window::Days30.to_string(), "30d");
        assert_eq!(Window::Older.to_string(), "6m+");
    }
}
