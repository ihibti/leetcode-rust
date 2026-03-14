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

pub fn run(root: &Path, force: bool, example_input: Option<&str>) -> Result<(), io::Error> {
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

    let test_code = match example_input {
        Some(input) if !input.trim().is_empty() => {
            let result = crate::parse_examples::parse_examples(input);
            for warning in &result.warnings {
                eprintln!("Warning: {warning}");
            }
            if result.examples.is_empty() && !result.warnings.is_empty() {
                eprintln!("Couldn't parse examples, starting with blank tests.");
                None
            } else if result.examples.is_empty() {
                None
            } else {
                Some(crate::parse_examples::generate_test_code(&result))
            }
        }
        _ => None,
    };

    let content = match test_code {
        Some(tests) => {
            format!(
                "use crate::types::*;\n\npub struct Solution;\n\n\
                 // Paste your impl Solution {{}} below\n\n{tests}")
        }
        None => SOLUTION_TEMPLATE.to_string(),
    };

    fs::write(&solution_path, content)?;
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
        run(dir.path(), false, None).unwrap();

        let content = fs::read_to_string(dir.path().join("src/solution.rs")).unwrap();
        assert_eq!(content, SOLUTION_TEMPLATE);
        assert!(dir.path().join(".solve_session").exists());
    }

    #[test]
    fn solve_refuses_overwrite_without_force() {
        let dir = setup_dir();
        let solution = dir.path().join("src/solution.rs");
        fs::write(&solution, "modified content").unwrap();

        let result = run(dir.path(), false, None);
        assert!(result.is_err());
    }

    #[test]
    fn solve_overwrites_with_force() {
        let dir = setup_dir();
        let solution = dir.path().join("src/solution.rs");
        fs::write(&solution, "modified content").unwrap();

        run(dir.path(), true, None).unwrap();
        let content = fs::read_to_string(&solution).unwrap();
        assert_eq!(content, SOLUTION_TEMPLATE);
    }

    #[test]
    fn solve_with_valid_examples() {
        let dir = setup_dir();
        let examples = "\
Example 1:

Input: nums = [2,7,11,15], target = 9
Output: [0,1]

Example 2:

Input: nums = [3,2,4], target = 6
Output: [1,2]";

        run(dir.path(), false, Some(examples)).unwrap();

        let content = fs::read_to_string(dir.path().join("src/solution.rs")).unwrap();
        assert!(content.contains("fn example_1()"));
        assert!(content.contains("fn example_2()"));
        assert!(content.contains("let nums = vec![2, 7, 11, 15];"));
        assert!(content.contains("let expected = vec![0, 1];"));
    }

    #[test]
    fn solve_with_garbage_examples() {
        let dir = setup_dir();

        run(dir.path(), false, Some("not a leetcode example")).unwrap();

        let content = fs::read_to_string(dir.path().join("src/solution.rs")).unwrap();
        assert!(content.contains("fn example()"));
        assert!(content.contains("// your tests here"));
    }

    #[test]
    fn solve_with_none_gives_clean_template() {
        let dir = setup_dir();
        run(dir.path(), false, None).unwrap();

        let content = fs::read_to_string(dir.path().join("src/solution.rs")).unwrap();
        assert_eq!(content, SOLUTION_TEMPLATE);
    }
}
