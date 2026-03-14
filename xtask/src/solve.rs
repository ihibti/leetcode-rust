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

fn assemble_template(data: &crate::leetcode::ProblemData) -> String {
    let parse_result = crate::parse_examples::parse_examples(&data.examples_text);
    let method_name = crate::leetcode::extract_method_name(&data.rust_snippet);

    for warning in &parse_result.warnings {
        eprintln!("Warning: {warning}");
    }

    let test_code = crate::parse_examples::generate_test_code(&parse_result, method_name.as_deref());

    format!(
        "use crate::types::*;\n\npub struct Solution;\n\n{}\n\n{test_code}",
        data.rust_snippet
    )
}

pub fn run(root: &Path, force: bool, url: Option<&str>) -> Result<(), io::Error> {
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

    let content = match url {
        Some(url) => {
            match crate::leetcode::extract_slug(url) {
                Some(slug) => {
                    eprintln!("Fetching problem: {slug}...");
                    match crate::leetcode::fetch_problem(&slug) {
                        Ok(data) => {
                            eprintln!("Got it: {}", data.title);
                            assemble_template(&data)
                        }
                        Err(e) => {
                            eprintln!("Warning: {e}");
                            eprintln!("Starting with blank template.");
                            SOLUTION_TEMPLATE.to_string()
                        }
                    }
                }
                None => {
                    eprintln!("Warning: not a valid LeetCode URL.");
                    eprintln!("Expected: https://leetcode.com/problems/<problem-name>/");
                    eprintln!("Starting with blank template.");
                    SOLUTION_TEMPLATE.to_string()
                }
            }
        }
        None => {
            println!("Tip: pass a LeetCode URL to auto-generate tests (cargo solve <url>)");
            SOLUTION_TEMPLATE.to_string()
        }
    };

    fs::write(&solution_path, &content)?;
    fs::write(&session_path, Utc::now().to_rfc3339())?;

    println!("Ready! Open src/solution.rs");
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
    fn solve_no_url_gives_template() {
        let dir = setup_dir();
        run(dir.path(), false, None).unwrap();

        let content = fs::read_to_string(dir.path().join("src/solution.rs")).unwrap();
        assert_eq!(content, SOLUTION_TEMPLATE);
    }

    #[test]
    fn solve_invalid_url_gives_template() {
        let dir = setup_dir();
        run(dir.path(), false, Some("https://example.com/not-leetcode")).unwrap();

        let content = fs::read_to_string(dir.path().join("src/solution.rs")).unwrap();
        assert_eq!(content, SOLUTION_TEMPLATE);
    }

    #[test]
    fn assemble_template_with_examples() {
        let data = crate::leetcode::ProblemData {
            slug: "two-sum".into(),
            title: "Two Sum".into(),
            examples_text: "\
Example 1:

Input: nums = [2,7,11,15], target = 9
Output: [0,1]

Example 2:

Input: nums = [3,2,4], target = 6
Output: [1,2]"
            .into(),
            rust_snippet: "impl Solution {\n    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {\n        \n    }\n}".into(),
        };

        let content = assemble_template(&data);
        assert!(content.contains("pub struct Solution;"));
        assert!(content.contains("pub fn two_sum"));
        assert!(content.contains("fn example_1()"));
        assert!(content.contains("fn example_2()"));
        assert!(content.contains("let result = Solution::two_sum(nums, target);"));
        assert!(content.contains("assert_eq!(result, expected);"));
        assert!(!content.contains("// TODO"));
    }

    #[test]
    fn assemble_template_with_bad_examples() {
        let data = crate::leetcode::ProblemData {
            slug: "test".into(),
            title: "Test".into(),
            examples_text: "garbage that wont parse".into(),
            rust_snippet: "impl Solution {\n    pub fn foo(x: i32) -> i32 {\n        \n    }\n}".into(),
        };

        let content = assemble_template(&data);
        assert!(content.contains("pub fn foo"));
        assert!(content.contains("fn example()"));
        assert!(content.contains("// your tests here"));
    }
}
