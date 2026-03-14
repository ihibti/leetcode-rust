use chrono::{DateTime, Utc};
use std::fs;
use std::io;
use std::path::Path;

use crate::solve::SOLUTION_TEMPLATE;

struct SessionMeta {
    difficulty: Option<String>,
    tags: Option<String>,
}

fn read_session_meta(session_path: &Path) -> SessionMeta {
    let Ok(content) = fs::read_to_string(session_path) else {
        return SessionMeta { difficulty: None, tags: None };
    };

    let difficulty = extract_field(&content, "difficulty");
    let tags = extract_tags_field(&content);

    SessionMeta { difficulty, tags }
}

fn extract_field(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = &json[start..];
    let end = rest.find('"')?;
    let value = &rest[..end];
    if value.is_empty() || value == "unknown" {
        None
    } else {
        Some(value.to_string())
    }
}

fn extract_tags_field(json: &str) -> Option<String> {
    let start = json.find("\"tags\":[")?;
    let rest = &json[start + "\"tags\":[".len()..];
    let end = rest.find(']')?;
    let arr = &rest[..end];
    if arr.is_empty() {
        return None;
    }
    let tags: Vec<&str> = arr.split(',')
        .map(|s| s.trim().trim_matches('"'))
        .filter(|s| !s.is_empty())
        .collect();
    if tags.is_empty() {
        None
    } else {
        Some(tags.join(","))
    }
}

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

    let meta = read_session_meta(&session_path);
    let difficulty = difficulty.or(meta.difficulty);
    let tags = tags.or(meta.tags);

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
    let Ok(raw) = fs::read_to_string(session_path) else {
        return "unknown".to_string();
    };
    let timestamp_str = extract_field(&raw, "timestamp")
        .unwrap_or_else(|| raw.trim().to_string());
    let Ok(start) = timestamp_str.parse::<DateTime<Utc>>() else {
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

    #[test]
    fn read_meta_from_session() {
        let dir = setup_dir();
        fs::write(
            dir.path().join(".solve_session"),
            r#"{"timestamp":"2026-01-01T00:00:00Z","slug":"two-sum","difficulty":"Easy","tags":["Array","Hash Table"]}"#,
        ).unwrap();

        let meta = read_session_meta(&dir.path().join(".solve_session"));
        assert_eq!(meta.difficulty, Some("Easy".into()));
        assert_eq!(meta.tags, Some("Array,Hash Table".into()));
    }

    #[test]
    fn read_meta_missing_file() {
        let dir = setup_dir();
        let meta = read_session_meta(&dir.path().join(".solve_session"));
        assert_eq!(meta.difficulty, None);
        assert_eq!(meta.tags, None);
    }

    #[test]
    fn read_meta_old_format() {
        let dir = setup_dir();
        fs::write(
            dir.path().join(".solve_session"),
            "2026-01-01T00:00:00Z",
        ).unwrap();

        let meta = read_session_meta(&dir.path().join(".solve_session"));
        assert_eq!(meta.difficulty, None);
        assert_eq!(meta.tags, None);
    }

    #[test]
    fn archive_uses_session_metadata() {
        let dir = setup_dir();
        fs::write(
            dir.path().join(".solve_session"),
            r#"{"timestamp":"2026-01-01T00:00:00Z","slug":"two-sum","difficulty":"Easy","tags":["Array","Hash Table"]}"#,
        ).unwrap();

        run(dir.path(), "two-sum", None, None, None).unwrap();

        let archived = fs::read_to_string(dir.path().join("archive/two_sum.rs")).unwrap();
        assert!(archived.contains("//! Difficulty: Easy"));
        assert!(archived.contains("//! Tags: Array,Hash Table"));
    }

    #[test]
    fn archive_cli_overrides_session() {
        let dir = setup_dir();
        fs::write(
            dir.path().join(".solve_session"),
            r#"{"timestamp":"2026-01-01T00:00:00Z","slug":"two-sum","difficulty":"Easy","tags":["Array"]}"#,
        ).unwrap();

        run(dir.path(), "two-sum", Some("hard".into()), Some("dp".into()), None).unwrap();

        let archived = fs::read_to_string(dir.path().join("archive/two_sum.rs")).unwrap();
        assert!(archived.contains("//! Difficulty: hard"));
        assert!(archived.contains("//! Tags: dp"));
    }

    #[test]
    fn archive_no_session_no_flags() {
        let dir = setup_dir();
        run(dir.path(), "test", None, None, None).unwrap();

        let archived = fs::read_to_string(dir.path().join("archive/test.rs")).unwrap();
        assert!(!archived.contains("//! Difficulty:"));
        assert!(!archived.contains("//! Tags:"));
    }
}
