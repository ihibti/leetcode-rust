use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Default)]
pub struct ProblemMeta {
    pub name: String,
    pub difficulty: String,
    pub tags: Vec<String>,
    pub rust_concepts: Vec<String>,
    pub date: String,
    pub time: String,
}

pub fn run(root: &Path) -> Result<(), io::Error> {
    let archive_dir = root.join("archive");
    if !archive_dir.exists() {
        println!("No problems archived yet. Solve your first problem and run `cargo archive`!");
        return Ok(());
    }

    let mut problems = Vec::new();
    for entry in fs::read_dir(&archive_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() || path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let content = fs::read_to_string(&path)?;
        problems.push(parse_meta(&content));
    }

    if problems.is_empty() {
        println!("No problems archived yet. Solve your first problem and run `cargo archive`!");
        return Ok(());
    }

    problems.sort_by(|a, b| a.date.cmp(&b.date));
    display(&problems);
    Ok(())
}

pub fn parse_meta(content: &str) -> ProblemMeta {
    let mut meta = ProblemMeta::default();
    for line in content.lines() {
        let Some(rest) = line.strip_prefix("//! ") else {
            if !line.starts_with("//!") {
                break;
            }
            continue;
        };
        if let Some(val) = rest.strip_prefix("Problem: ") {
            meta.name = val.to_string();
        } else if let Some(val) = rest.strip_prefix("Difficulty: ") {
            meta.difficulty = val.to_string();
        } else if let Some(val) = rest.strip_prefix("Tags: ") {
            meta.tags = val.split(',').map(|s| s.trim().to_string()).collect();
        } else if let Some(val) = rest.strip_prefix("Rust concepts: ") {
            meta.rust_concepts = val.split(',').map(|s| s.trim().to_string()).collect();
        } else if let Some(val) = rest.strip_prefix("Date: ") {
            meta.date = val.to_string();
        } else if let Some(val) = rest.strip_prefix("Time: ") {
            meta.time = val.to_string();
        }
    }
    meta
}

fn display(problems: &[ProblemMeta]) {
    println!("LeetCode Progress — {} problems solved\n", problems.len());

    let mut diff_counts: HashMap<&str, usize> = HashMap::new();
    let mut concept_counts: HashMap<String, usize> = HashMap::new();

    for p in problems {
        if !p.difficulty.is_empty() {
            let key = match p.difficulty.as_str() {
                "easy" => "Easy",
                "medium" => "Medium",
                "hard" => "Hard",
                other => other,
            };
            *diff_counts.entry(key).or_default() += 1;
        }
        for c in &p.rust_concepts {
            *concept_counts.entry(c.clone()).or_default() += 1;
        }
    }

    println!("Difficulty:");
    for label in &["Easy", "Medium", "Hard"] {
        let count = diff_counts.get(label).copied().unwrap_or(0);
        if count > 0 {
            let bar: String = "\u{2588}".repeat(count);
            println!("  {label:<8} {bar} {count}");
        } else {
            println!("  {label:<8} \u{25CB} 0");
        }
    }

    if !concept_counts.is_empty() {
        println!("\nRust Concepts:");
        let mut concepts: Vec<_> = concept_counts.iter().collect();
        concepts.sort_by(|a, b| b.1.cmp(a.1));
        for (concept, count) in concepts {
            let bar: String = "\u{2588}".repeat(*count);
            println!("  {concept:<16} {bar} {count}");
        }
    }

    println!("\nRecent:");
    for p in problems.iter().rev().take(5) {
        let time = if p.time.is_empty() { "?" } else { &p.time };
        let diff = if p.difficulty.is_empty() {
            String::new()
        } else {
            format!("({})", p.difficulty)
        };
        println!("  {:<20} {:<6} {:<10} {}", p.name, time, diff, p.date);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_metadata() {
        let content = "//! Problem: two-sum\n//! Difficulty: easy\n//! Tags: array, hash-map\n//! Rust concepts: iterators, entry-api\n//! Date: 2026-03-14\n//! Time: 12m\n\nuse crate::types::*;\n";
        let meta = parse_meta(content);
        assert_eq!(meta.name, "two-sum");
        assert_eq!(meta.difficulty, "easy");
        assert_eq!(meta.tags, vec!["array", "hash-map"]);
        assert_eq!(meta.rust_concepts, vec!["iterators", "entry-api"]);
        assert_eq!(meta.date, "2026-03-14");
        assert_eq!(meta.time, "12m");
    }

    #[test]
    fn parse_partial_metadata() {
        let content = "//! Problem: test\n//! Date: 2026-01-01\n\ncode here\n";
        let meta = parse_meta(content);
        assert_eq!(meta.name, "test");
        assert_eq!(meta.date, "2026-01-01");
        assert!(meta.difficulty.is_empty());
        assert!(meta.tags.is_empty());
    }
}
