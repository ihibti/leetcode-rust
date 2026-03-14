use chrono::{DateTime, Utc};
use std::fs;
use std::io;
use std::path::Path;

use crate::solve::SOLUTION_TEMPLATE;

pub fn run(
    root: &Path,
    name: &str,
    difficulty: Option<String>,
    tags: Option<String>,
    rust_concepts: Option<String>,
) -> Result<(), io::Error> {
    let solution_path = root.join("src/solution.rs");
    let archive_dir = root.join("archive");
    let session_path = root.join(".solve_session");

    let content = fs::read_to_string(&solution_path)?;
    if content.trim() == SOLUTION_TEMPLATE.trim() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "solution.rs matches template — nothing to archive",
        ));
    }

    let filename = normalize_name(name);
    let archive_path = archive_dir.join(format!("{filename}.rs"));
    if archive_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("archive/{filename}.rs already exists"),
        ));
    }

    let time_str = calculate_time(&session_path);
    let date_str = Utc::now().format("%Y-%m-%d").to_string();

    let mut header = format!("//! Problem: {name}\n");
    if let Some(d) = &difficulty {
        header.push_str(&format!("//! Difficulty: {d}\n"));
    }
    if let Some(t) = &tags {
        header.push_str(&format!("//! Tags: {t}\n"));
    }
    if let Some(r) = &rust_concepts {
        header.push_str(&format!("//! Rust concepts: {r}\n"));
    }
    header.push_str(&format!("//! Date: {date_str}\n"));
    header.push_str(&format!("//! Time: {time_str}\n"));
    header.push('\n');

    fs::create_dir_all(&archive_dir)?;
    fs::write(&archive_path, format!("{header}{content}"))?;
    fs::write(&solution_path, SOLUTION_TEMPLATE)?;
    let _ = fs::remove_file(&session_path);

    println!("Saved to archive/{filename}.rs — nice work!");
    Ok(())
}

pub fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(['-', ' '], "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

fn calculate_time(session_path: &Path) -> String {
    let Ok(timestamp_str) = fs::read_to_string(session_path) else {
        return "unknown".to_string();
    };
    let Ok(start) = timestamp_str.trim().parse::<DateTime<Utc>>() else {
        return "unknown".to_string();
    };
    let elapsed = Utc::now().signed_duration_since(start);
    let total_minutes = elapsed.num_minutes();
    if total_minutes < 60 {
        format!("{total_minutes}m")
    } else {
        format!("{}h{}m", total_minutes / 60, total_minutes % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(
            dir.path().join("src/solution.rs"),
            format!(
                "{}\nimpl Solution {{\n    pub fn two_sum() {{}}\n}}",
                SOLUTION_TEMPLATE
            ),
        )
        .unwrap();
        dir
    }

    #[test]
    fn normalize_hyphen() {
        assert_eq!(normalize_name("two-sum"), "two_sum");
    }

    #[test]
    fn normalize_mixed() {
        assert_eq!(normalize_name("3Sum Closest"), "3sum_closest");
    }

    #[test]
    fn archive_creates_file() {
        let dir = setup_dir();
        run(dir.path(), "two-sum", Some("easy".into()), None, None).unwrap();

        let archived = fs::read_to_string(dir.path().join("archive/two_sum.rs")).unwrap();
        assert!(archived.starts_with("//! Problem: two-sum\n"));
        assert!(archived.contains("//! Difficulty: easy\n"));
        assert!(archived.contains("impl Solution"));
    }

    #[test]
    fn archive_resets_template() {
        let dir = setup_dir();
        run(dir.path(), "test", None, None, None).unwrap();

        let content = fs::read_to_string(dir.path().join("src/solution.rs")).unwrap();
        assert_eq!(content, SOLUTION_TEMPLATE);
    }

    #[test]
    fn archive_rejects_collision() {
        let dir = setup_dir();
        run(dir.path(), "test", None, None, None).unwrap();

        fs::write(dir.path().join("src/solution.rs"), "modified again").unwrap();

        let result = run(dir.path(), "test", None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn archive_rejects_empty() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/solution.rs"), SOLUTION_TEMPLATE).unwrap();

        let result = run(dir.path(), "test", None, None, None);
        assert!(result.is_err());
    }
}
