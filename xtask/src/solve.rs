use chrono::Utc;
use std::fs;
use std::io;
use std::path::Path;

pub const SOLUTION_TEMPLATE: &str = r#"use crate::types::*;

pub struct Solution;

// Paste your impl Solution { ... } below

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{list, tree};

    #[test]
    fn example() {
        // your tests here
    }
}
"#;

pub fn run(root: &Path, force: bool) -> Result<(), io::Error> {
    let solution_path = root.join("src/solution.rs");
    let session_path = root.join(".solve_session");

    if solution_path.exists() && !force {
        let content = fs::read_to_string(&solution_path)?;
        if content.trim() != SOLUTION_TEMPLATE.trim() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "solution.rs has unsaved work. Use `cargo solve --force` to overwrite,\n\
                 or run `cargo archive <name>` first to save your solution.",
            ));
        }
    }

    fs::write(&solution_path, SOLUTION_TEMPLATE)?;
    fs::write(&session_path, Utc::now().to_rfc3339())?;

    println!("Ready! Open src/solution.rs and paste your impl Solution.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        dir
    }

    #[test]
    fn solve_creates_template() {
        let dir = setup_dir();
        run(dir.path(), false).unwrap();

        let content = fs::read_to_string(dir.path().join("src/solution.rs")).unwrap();
        assert_eq!(content, SOLUTION_TEMPLATE);
        assert!(dir.path().join(".solve_session").exists());
    }

    #[test]
    fn solve_refuses_overwrite_without_force() {
        let dir = setup_dir();
        let solution = dir.path().join("src/solution.rs");
        fs::write(&solution, "modified content").unwrap();

        let result = run(dir.path(), false);
        assert!(result.is_err());
    }

    #[test]
    fn solve_overwrites_with_force() {
        let dir = setup_dir();
        let solution = dir.path().join("src/solution.rs");
        fs::write(&solution, "modified content").unwrap();

        run(dir.path(), true).unwrap();
        let content = fs::read_to_string(&solution).unwrap();
        assert_eq!(content, SOLUTION_TEMPLATE);
    }
}
