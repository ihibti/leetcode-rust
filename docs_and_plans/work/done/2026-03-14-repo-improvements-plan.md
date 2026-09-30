# Repo Improvements — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a bootstrap shell script (`./lc`), optional test generation from LeetCode examples in `cargo solve`, and documentation for 42 school students learning Rust from C.

**Architecture:** Three independent layers: (1) `./lc` bash script for pre-cargo setup/help/reset, (2) `parse_examples.rs` pure-function parser module called from `solve.rs` for best-effort test generation, (3) markdown documentation. The parser is the riskiest piece — built TDD with exhaustive edge case coverage.

**Tech Stack:** Rust 2024 edition, bash, standard library only (no new crate deps)

**Spec:** `docs/plans/2026-03-14-repo-improvements-design.md`

---

## Chunk 1: Parser Module — Value Parsing

The parser is a new file `xtask/src/parse_examples.rs`. It contains pure functions with no I/O. This chunk covers the lowest-level building block: converting a single LeetCode value string into a Rust code string.

### Task 1: Value Parsing Core

**Files:**
- Create: `xtask/src/parse_examples.rs`
- Modify: `xtask/src/main.rs` (add `mod parse_examples;`)

- [ ] **Step 1: Register the module**

Add to `xtask/src/main.rs` after line 4 (`mod solve;`):

```rust
mod parse_examples;
```

- [ ] **Step 2: Write failing tests for value parsing**

Create `xtask/src/parse_examples.rs` with only the test module and a function stub that always returns `None`:

```rust
pub fn parse_value(s: &str) -> Option<String> {
    None
}

fn split_top_level(s: &str, delimiter: char) -> Vec<String> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_integer() {
        assert_eq!(parse_value("5"), Some("5".into()));
    }

    #[test]
    fn value_negative_integer() {
        assert_eq!(parse_value("-3"), Some("-3".into()));
    }

    #[test]
    fn value_zero() {
        assert_eq!(parse_value("0"), Some("0".into()));
    }

    #[test]
    fn value_float() {
        assert_eq!(parse_value("2.50000"), Some("2.5f64".into()));
    }

    #[test]
    fn value_negative_float() {
        assert_eq!(parse_value("-1.5"), Some("-1.5f64".into()));
    }

    #[test]
    fn value_zero_float() {
        assert_eq!(parse_value("0.0"), Some("0.0f64".into()));
    }

    #[test]
    fn value_bool_true() {
        assert_eq!(parse_value("true"), Some("true".into()));
    }

    #[test]
    fn value_bool_false() {
        assert_eq!(parse_value("false"), Some("false".into()));
    }

    #[test]
    fn value_string() {
        assert_eq!(
            parse_value("\"abc\""),
            Some("String::from(\"abc\")".into())
        );
    }

    #[test]
    fn value_empty_string() {
        assert_eq!(
            parse_value("\"\""),
            Some("String::from(\"\")".into())
        );
    }

    #[test]
    fn value_string_with_spaces() {
        assert_eq!(
            parse_value("\"hello world\""),
            Some("String::from(\"hello world\")".into())
        );
    }

    #[test]
    fn value_simple_array() {
        assert_eq!(
            parse_value("[1,2,3]"),
            Some("vec![1, 2, 3]".into())
        );
    }

    #[test]
    fn value_empty_array() {
        assert_eq!(parse_value("[]"), Some("vec![]".into()));
    }

    #[test]
    fn value_single_element_array() {
        assert_eq!(parse_value("[1]"), Some("vec![1]".into()));
    }

    #[test]
    fn value_array_with_negatives() {
        assert_eq!(
            parse_value("[-1,0,1]"),
            Some("vec![-1, 0, 1]".into())
        );
    }

    #[test]
    fn value_nested_array() {
        assert_eq!(
            parse_value("[[1,2],[3,4]]"),
            Some("vec![vec![1, 2], vec![3, 4]]".into())
        );
    }

    #[test]
    fn value_string_array() {
        assert_eq!(
            parse_value("[\"a\",\"b\"]"),
            Some("vec![String::from(\"a\"), String::from(\"b\")]".into())
        );
    }

    #[test]
    fn value_deeply_nested_array() {
        assert_eq!(
            parse_value("[[1,2,3],[4,5,6],[7,8,9]]"),
            Some("vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]".into())
        );
    }

    #[test]
    fn value_empty_input() {
        assert_eq!(parse_value(""), None);
    }

    #[test]
    fn value_whitespace_trimmed() {
        assert_eq!(parse_value("  5  "), Some("5".into()));
    }

    #[test]
    fn value_array_with_spaces() {
        assert_eq!(
            parse_value("[1, 2, 3]"),
            Some("vec![1, 2, 3]".into())
        );
    }

    #[test]
    fn split_simple_comma() {
        assert_eq!(
            split_top_level("1,2,3", ','),
            vec!["1", "2", "3"]
        );
    }

    #[test]
    fn split_respects_brackets() {
        assert_eq!(
            split_top_level("[1,2],[3,4]", ','),
            vec!["[1,2]", "[3,4]"]
        );
    }

    #[test]
    fn split_respects_strings() {
        assert_eq!(
            split_top_level("\"a,b\",\"c\"", ','),
            vec!["\"a,b\"", "\"c\""]
        );
    }

    #[test]
    fn split_empty() {
        let result: Vec<String> = split_top_level("", ',');
        assert!(result.is_empty() || result == vec![""]);
    }
}
```

- [ ] **Step 3: Run tests — verify they fail**

Run: `cargo test -p xtask parse_examples -- --nocapture 2>&1 | head -40`

Expected: all tests FAIL (stubs return `None` / empty)

- [ ] **Step 4: Implement `split_top_level`**

This is the core utility: splits a string on a delimiter character, but only when not inside brackets `[]` or strings `""`. Used by both array parsing and parameter parsing.

Replace the `split_top_level` stub in `xtask/src/parse_examples.rs`:

```rust
fn split_top_level(s: &str, delimiter: char) -> Vec<String> {
    let mut result = Vec::new();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut current = String::new();

    for ch in s.chars() {
        if ch == '"' {
            in_string = !in_string;
            current.push(ch);
        } else if in_string {
            current.push(ch);
        } else if ch == '[' {
            depth += 1;
            current.push(ch);
        } else if ch == ']' {
            depth -= 1;
            current.push(ch);
        } else if ch == delimiter && depth == 0 {
            result.push(current.trim().to_string());
            current = String::new();
        } else {
            current.push(ch);
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() || !result.is_empty() {
        result.push(trimmed);
    }

    result
}
```

- [ ] **Step 5: Implement `parse_value`**

Replace the `parse_value` stub:

```rust
pub fn parse_value(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    if s == "true" || s == "false" {
        return Some(s.to_string());
    }

    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        let inner = &s[1..s.len() - 1];
        return Some(format!("String::from(\"{inner}\")"));
    }

    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1].trim();
        if inner.is_empty() {
            return Some("vec![]".to_string());
        }
        let elements = split_top_level(inner, ',');
        let parsed: Vec<String> = elements
            .iter()
            .map(|e| parse_value(e))
            .collect::<Option<Vec<_>>>()?;
        return Some(format!("vec![{}]", parsed.join(", ")));
    }

    if s.contains('.') {
        let val: f64 = s.parse().ok()?;
        return Some(format!("{val:?}f64"));
    }

    let _: i64 = s.parse().ok()?;
    Some(s.to_string())
}
```

- [ ] **Step 6: Run tests — verify they pass**

Run: `cargo test -p xtask parse_examples`

Expected: all tests PASS

- [ ] **Step 7: Commit**

```bash
git add xtask/src/parse_examples.rs xtask/src/main.rs
git commit -m "Add parse_examples module with value parsing (TDD)"
```

---

## Chunk 2: Parser Module — Example Block Parsing & Code Generation

Builds on chunk 1. Adds parsing of `Input:` / `Output:` lines, splitting example blocks, and generating Rust test code.

### Task 2: Input Line Parsing

**Files:**
- Modify: `xtask/src/parse_examples.rs`

- [ ] **Step 1: Write failing tests for input line parsing**

Append to the `tests` module in `xtask/src/parse_examples.rs`:

```rust
    #[test]
    fn input_single_param() {
        let result = parse_input_line("nums = [1,2,3]").unwrap();
        assert_eq!(result, vec![
            ("nums".into(), "vec![1, 2, 3]".into()),
        ]);
    }

    #[test]
    fn input_multiple_params() {
        let result = parse_input_line("nums1 = [1,3], nums2 = [2]").unwrap();
        assert_eq!(result, vec![
            ("nums1".into(), "vec![1, 3]".into()),
            ("nums2".into(), "vec![2]".into()),
        ]);
    }

    #[test]
    fn input_scalar_params() {
        let result = parse_input_line("x = 5, y = 10").unwrap();
        assert_eq!(result, vec![
            ("x".into(), "5".into()),
            ("y".into(), "10".into()),
        ]);
    }

    #[test]
    fn input_string_param() {
        let result = parse_input_line("s = \"abc\"").unwrap();
        assert_eq!(result, vec![
            ("s".into(), "String::from(\"abc\")".into()),
        ]);
    }

    #[test]
    fn input_underscore_name() {
        let result = parse_input_line("linked_list = [1,2,3]").unwrap();
        assert_eq!(result, vec![
            ("linked_list".into(), "vec![1, 2, 3]".into()),
        ]);
    }

    #[test]
    fn input_mixed_types() {
        let result = parse_input_line("nums = [1,2,3], target = 6").unwrap();
        assert_eq!(result, vec![
            ("nums".into(), "vec![1, 2, 3]".into()),
            ("target".into(), "6".into()),
        ]);
    }

    #[test]
    fn input_no_spaces_around_equals() {
        let result = parse_input_line("n=5").unwrap();
        assert_eq!(result, vec![
            ("n".into(), "5".into()),
        ]);
    }

    #[test]
    fn input_empty() {
        let result = parse_input_line("");
        assert!(result.is_none() || result.unwrap().is_empty());
    }

    #[test]
    fn input_string_with_comma() {
        let result = parse_input_line("s = \"a,b\", t = \"c\"").unwrap();
        assert_eq!(result, vec![
            ("s".into(), "String::from(\"a,b\")".into()),
            ("t".into(), "String::from(\"c\")".into()),
        ]);
    }
```

- [ ] **Step 2: Add `parse_input_line` stub**

Add above the test module in `xtask/src/parse_examples.rs`:

```rust
pub fn parse_input_line(s: &str) -> Option<Vec<(String, String)>> {
    None
}
```

- [ ] **Step 3: Run tests — verify new tests fail**

Run: `cargo test -p xtask parse_examples::tests::input_`

Expected: all `input_*` tests FAIL

- [ ] **Step 4: Implement `parse_input_line`**

Replace the stub. The approach: find param boundaries by looking for `, name = ` or `, name=` patterns at bracket-depth 0, then parse each segment as `name = value`.

```rust
pub fn parse_input_line(s: &str) -> Option<Vec<(String, String)>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let mut params = Vec::new();
    let segments = split_param_assignments(s);

    for segment in &segments {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }

        let eq_pos = segment.find('=')
            .or_else(|| segment.find(" = "))?;

        let (name_part, value_part) = if segment[eq_pos..].starts_with("= ") || segment[eq_pos..].starts_with('=') {
            let name = segment[..eq_pos].trim();
            let value = segment[eq_pos + 1..].trim();
            let value = value.trim_start_matches(' ');
            (name, value)
        } else {
            return None;
        };

        let rust_value = parse_value(value_part)?;
        params.push((name_part.to_string(), rust_value));
    }

    if params.is_empty() {
        None
    } else {
        Some(params)
    }
}

fn split_param_assignments(s: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut depth = 0i32;
    let mut in_string = false;
    let mut current = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        if ch == '"' {
            in_string = !in_string;
            current.push(ch);
            i += 1;
        } else if in_string {
            current.push(ch);
            i += 1;
        } else if ch == '[' {
            depth += 1;
            current.push(ch);
            i += 1;
        } else if ch == ']' {
            depth -= 1;
            current.push(ch);
            i += 1;
        } else if ch == ',' && depth == 0 {
            let rest = &s[i + 1..].trim_start();
            let looks_like_param = rest.contains('=')
                && rest.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_');
            if looks_like_param {
                segments.push(current.trim().to_string());
                current = String::new();
                i += 1;
            } else {
                current.push(ch);
                i += 1;
            }
        } else {
            current.push(ch);
            i += 1;
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        segments.push(trimmed);
    }

    segments
}
```

- [ ] **Step 5: Run tests — verify they pass**

Run: `cargo test -p xtask parse_examples`

Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git add xtask/src/parse_examples.rs
git commit -m "Add input line parsing with param boundary detection"
```

### Task 3: Example Block Parsing & Code Generation

**Files:**
- Modify: `xtask/src/parse_examples.rs`

- [ ] **Step 1: Write failing tests for full example parsing and code generation**

Append to the `tests` module:

```rust
    #[test]
    fn parse_single_example() {
        let input = "\
Example 1:

Input: nums = [2,7,11,15], target = 9
Output: [0,1]
Explanation: Because nums[0] + nums[1] == 9, we return [0, 1].";

        let result = parse_examples(input);
        assert_eq!(result.examples.len(), 1);
        assert_eq!(result.warnings.len(), 0);
        assert_eq!(result.examples[0].params, vec![
            ("nums".into(), "vec![2, 7, 11, 15]".into()),
            ("target".into(), "9".into()),
        ]);
        assert_eq!(result.examples[0].output, "vec![0, 1]");
    }

    #[test]
    fn parse_two_examples() {
        let input = "\
Example 1:

Input: nums1 = [1,3], nums2 = [2]
Output: 2.00000
Explanation: merged array = [1,2,3] and median is 2.

Example 2:

Input: nums1 = [1,2], nums2 = [3,4]
Output: 2.50000
Explanation: merged array = [1,2,3,4] and median is (2 + 3) / 2 = 2.5.";

        let result = parse_examples(input);
        assert_eq!(result.examples.len(), 2);
        assert_eq!(result.examples[0].output, "2.0f64");
        assert_eq!(result.examples[1].output, "2.5f64");
    }

    #[test]
    fn parse_no_explanation() {
        let input = "\
Example 1:

Input: n = 5
Output: true";

        let result = parse_examples(input);
        assert_eq!(result.examples.len(), 1);
        assert_eq!(result.examples[0].output, "true");
    }

    #[test]
    fn parse_three_examples() {
        let input = "\
Example 1:

Input: x = 121
Output: true

Example 2:

Input: x = -121
Output: false

Example 3:

Input: x = 10
Output: false";

        let result = parse_examples(input);
        assert_eq!(result.examples.len(), 3);
    }

    #[test]
    fn parse_multiline_input() {
        let input = "\
Example 1:

Input: nums = [3,2,4]
target = 6
Output: [1,2]";

        let result = parse_examples(input);
        assert_eq!(result.examples.len(), 1);
        assert_eq!(result.examples[0].params.len(), 2);
    }

    #[test]
    fn parse_empty_input() {
        let result = parse_examples("");
        assert!(result.examples.is_empty());
    }

    #[test]
    fn parse_garbage() {
        let result = parse_examples("this is not a leetcode example at all");
        assert!(result.examples.is_empty());
    }

    #[test]
    fn parse_partial_failure() {
        let input = "\
Example 1:

Input: nums = [1,2,3]
Output: 6

Example 2:

Input: ???weird???
Output: ???";

        let result = parse_examples(input);
        assert_eq!(result.examples.len(), 1);
        assert!(result.warnings.len() >= 1);
    }

    #[test]
    fn parse_input_missing_output() {
        let input = "\
Example 1:

Input: n = 5";

        let result = parse_examples(input);
        assert!(result.examples.is_empty());
        assert!(result.warnings.len() >= 1);
    }

    #[test]
    fn parse_string_output() {
        let input = "\
Example 1:

Input: s = \"abc\"
Output: \"cba\"";

        let result = parse_examples(input);
        assert_eq!(result.examples.len(), 1);
        assert_eq!(result.examples[0].output, "String::from(\"cba\")");
    }

    #[test]
    fn generate_basic_test_code() {
        let input = "\
Example 1:

Input: nums = [2,7,11,15], target = 9
Output: [0,1]";

        let result = parse_examples(input);
        let code = generate_test_code(&result);

        assert!(code.contains("#[cfg(test)]"));
        assert!(code.contains("fn example_1()"));
        assert!(code.contains("let nums = vec![2, 7, 11, 15];"));
        assert!(code.contains("let target = 9;"));
        assert!(code.contains("let expected = vec![0, 1];"));
        assert!(code.contains("// TODO"));
    }

    #[test]
    fn generate_multiple_tests() {
        let input = "\
Example 1:

Input: x = 121
Output: true

Example 2:

Input: x = -121
Output: false";

        let result = parse_examples(input);
        let code = generate_test_code(&result);

        assert!(code.contains("fn example_1()"));
        assert!(code.contains("fn example_2()"));
    }

    #[test]
    fn generate_empty_gives_default() {
        let result = parse_examples("");
        let code = generate_test_code(&result);

        assert!(code.contains("fn example()"));
        assert!(code.contains("// your tests here"));
    }
```

- [ ] **Step 2: Add data structures and stubs**

Add above the test module in `xtask/src/parse_examples.rs`:

```rust
pub struct Example {
    pub params: Vec<(String, String)>,
    pub output: String,
}

pub struct ParseResult {
    pub examples: Vec<Example>,
    pub warnings: Vec<String>,
}

pub fn parse_examples(_input: &str) -> ParseResult {
    ParseResult {
        examples: vec![],
        warnings: vec![],
    }
}

pub fn generate_test_code(_result: &ParseResult) -> String {
    String::new()
}
```

- [ ] **Step 3: Run tests — verify new tests fail**

Run: `cargo test -p xtask parse_examples::tests::parse_ -- --nocapture 2>&1 | head -40`

Expected: new tests FAIL

- [ ] **Step 4: Implement `parse_examples`**

Replace the stub:

```rust
pub fn parse_examples(input: &str) -> ParseResult {
    let mut examples = Vec::new();
    let mut warnings = Vec::new();

    let input = input.trim();
    if input.is_empty() {
        return ParseResult { examples, warnings };
    }

    let blocks = split_example_blocks(input);
    if blocks.is_empty() {
        return ParseResult { examples, warnings };
    }

    for (idx, block) in blocks.iter().enumerate() {
        match parse_single_example(block) {
            Some(example) => examples.push(example),
            None => warnings.push(format!("Skipped example {} (couldn't parse)", idx + 1)),
        }
    }

    ParseResult { examples, warnings }
}

fn split_example_blocks(input: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut found_any = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Example ") && trimmed.ends_with(':') {
            if found_any && !current.trim().is_empty() {
                blocks.push(current.trim().to_string());
            }
            current = String::new();
            found_any = true;
        } else if found_any {
            current.push_str(line);
            current.push('\n');
        }
    }

    if found_any && !current.trim().is_empty() {
        blocks.push(current.trim().to_string());
    }

    blocks
}

fn parse_single_example(block: &str) -> Option<Example> {
    let mut input_lines = Vec::new();
    let mut output_line = None;
    let mut in_input = false;

    for line in block.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Input:") {
            in_input = true;
            let rest = trimmed.strip_prefix("Input:")?.trim();
            if !rest.is_empty() {
                input_lines.push(rest.to_string());
            }
        } else if trimmed.starts_with("Output:") {
            in_input = false;
            let rest = trimmed.strip_prefix("Output:")?.trim();
            output_line = Some(rest.to_string());
        } else if trimmed.starts_with("Explanation:") {
            in_input = false;
        } else if in_input && !trimmed.is_empty() {
            input_lines.push(trimmed.to_string());
        }
    }

    let output_str = output_line?;
    let output = parse_value(&output_str)?;

    let combined_input = input_lines.join(", ");
    let params = parse_input_line(&combined_input)?;

    if params.is_empty() {
        return None;
    }

    Some(Example { params, output })
}
```

- [ ] **Step 5: Implement `generate_test_code`**

Replace the stub:

```rust
pub fn generate_test_code(result: &ParseResult) -> String {
    let mut code = String::new();

    code.push_str("#[cfg(test)]\nmod tests {\n");
    code.push_str("    use super::*;\n");
    code.push_str("    use crate::{list, tree};\n");

    if result.examples.is_empty() {
        code.push_str("\n    #[test]\n");
        code.push_str("    fn example() {\n");
        code.push_str("        // your tests here\n");
        code.push_str("    }\n");
    } else {
        for (i, example) in result.examples.iter().enumerate() {
            code.push_str(&format!("\n    #[test]\n"));
            code.push_str(&format!("    fn example_{}() {{\n", i + 1));

            for (name, value) in &example.params {
                code.push_str(&format!("        let {name} = {value};\n"));
            }
            code.push_str(&format!("        let expected = {};\n", example.output));

            let param_names: Vec<&str> = example.params.iter().map(|(n, _)| n.as_str()).collect();
            let args = param_names.join(", ");
            code.push_str(&format!(
                "        // TODO: uncomment and replace method_name with your method\n"
            ));
            code.push_str(&format!(
                "        // let result = Solution::method_name({args});\n"
            ));
            code.push_str("        // assert_eq!(result, expected);\n");
            code.push_str("    }\n");
        }
    }

    code.push_str("}\n");
    code
}
```

- [ ] **Step 6: Run tests — verify they pass**

Run: `cargo test -p xtask parse_examples`

Expected: all tests PASS

- [ ] **Step 7: Commit**

```bash
git add xtask/src/parse_examples.rs
git commit -m "Add example block parsing and test code generation"
```

### Task 4: Edge Case Hardening

**Files:**
- Modify: `xtask/src/parse_examples.rs`

- [ ] **Step 1: Write edge case tests**

Append to the `tests` module:

```rust
    #[test]
    fn value_trailing_comma_in_array() {
        assert_eq!(
            parse_value("[1,2,3,]"),
            Some("vec![1, 2, 3]".into())
        );
    }

    #[test]
    fn value_large_integer() {
        assert_eq!(
            parse_value("2147483647"),
            Some("2147483647".into())
        );
    }

    #[test]
    fn value_large_negative() {
        assert_eq!(
            parse_value("-2147483648"),
            Some("-2147483648".into())
        );
    }

    #[test]
    fn value_nested_empty_arrays() {
        assert_eq!(
            parse_value("[[]]"),
            Some("vec![vec![]]".into())
        );
    }

    #[test]
    fn value_float_one() {
        assert_eq!(
            parse_value("1.00000"),
            Some("1.0f64".into())
        );
    }

    #[test]
    fn parse_whitespace_variations() {
        let input = "\
Example 1:

Input:  nums =  [1, 2, 3] , target = 6
Output:  [0, 1]  ";

        let result = parse_examples(input);
        assert_eq!(result.examples.len(), 1);
    }

    #[test]
    fn parse_only_explanation() {
        let input = "\
Example 1:

Explanation: This should not parse.";

        let result = parse_examples(input);
        assert!(result.examples.is_empty());
    }

    #[test]
    fn parse_boolean_output() {
        let input = "\
Example 1:

Input: s = \"()\"
Output: true

Example 2:

Input: s = \"(]\"
Output: false";

        let result = parse_examples(input);
        assert_eq!(result.examples.len(), 2);
        assert_eq!(result.examples[0].output, "true");
        assert_eq!(result.examples[1].output, "false");
    }

    #[test]
    fn parse_nested_array_output() {
        let input = "\
Example 1:

Input: matrix = [[1,2,3],[4,5,6],[7,8,9]]
Output: [[7,4,1],[8,5,2],[9,6,3]]";

        let result = parse_examples(input);
        assert_eq!(result.examples.len(), 1);
        assert_eq!(
            result.examples[0].output,
            "vec![vec![7, 4, 1], vec![8, 5, 2], vec![9, 6, 3]]"
        );
    }

    #[test]
    fn generate_preserves_warnings() {
        let result = ParseResult {
            examples: vec![],
            warnings: vec!["something went wrong".into()],
        };
        let code = generate_test_code(&result);
        assert!(code.contains("fn example()"));
    }
```

- [ ] **Step 2: Run tests — verify edge cases pass or identify fixes**

Run: `cargo test -p xtask parse_examples`

If `value_trailing_comma_in_array` fails, update `parse_value` to handle trailing commas: filter out empty elements in the `split_top_level` result before mapping through `parse_value`:

```rust
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1].trim();
        if inner.is_empty() {
            return Some("vec![]".to_string());
        }
        let elements = split_top_level(inner, ',');
        let parsed: Vec<String> = elements
            .iter()
            .filter(|e| !e.trim().is_empty())
            .map(|e| parse_value(e))
            .collect::<Option<Vec<_>>>()?;
        return Some(format!("vec![{}]", parsed.join(", ")));
    }
```

- [ ] **Step 3: Verify all tests pass**

Run: `cargo test -p xtask parse_examples`

Expected: all tests PASS

- [ ] **Step 4: Commit**

```bash
git add xtask/src/parse_examples.rs
git commit -m "Add edge case tests for parser"
```

---

## Chunk 3: Solve Integration & Setup Deprecation

Connects the parser to `cargo solve` and deprecates `cargo setup` in favor of `./lc setup`.

### Task 5: Update `solve::run` Signature

**Files:**
- Modify: `xtask/src/solve.rs`
- Modify: `xtask/src/main.rs`

- [ ] **Step 1: Update existing tests to pass `None` as third argument**

In `xtask/src/solve.rs`, update all three existing test calls:

Change `run(dir.path(), false).unwrap();` to `run(dir.path(), false, None).unwrap();`
Change `run(dir.path(), false);` to `run(dir.path(), false, None);`
Change `run(dir.path(), true).unwrap();` to `run(dir.path(), true, None).unwrap();`

- [ ] **Step 2: Add new integration tests**

Append to the `tests` module in `xtask/src/solve.rs`:

```rust
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
```

- [ ] **Step 3: Update `run` function signature and implementation**

Replace the `run` function in `xtask/src/solve.rs`:

```rust
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
```

- [ ] **Step 4: Update `main.rs` call site with TTY detection and stdin reading**

In `xtask/src/main.rs`, update the `Solve` arm. Replace:

```rust
        Command::Solve { force } => solve::run(&root, force),
```

With:

```rust
        Command::Solve { force } => {
            use std::io::{self, IsTerminal, Read};

            let example_input = if io::stdin().is_terminal() {
                println!("Paste LeetCode examples to auto-generate tests?");
                println!("(paste examples, then Ctrl+D to confirm — or just press Enter to skip)\n");

                let mut input = String::new();
                let mut first_line = String::new();
                if io::stdin().read_line(&mut first_line).is_ok() {
                    if first_line.trim().is_empty() {
                        None
                    } else {
                        input.push_str(&first_line);
                        let _ = io::stdin().read_to_string(&mut input);
                        Some(input)
                    }
                } else {
                    None
                }
            } else {
                None
            };

            solve::run(&root, force, example_input.as_deref())
        }
```

- [ ] **Step 5: Run all tests**

Run: `cargo test -p xtask`

Expected: all tests PASS (existing + new)

- [ ] **Step 6: Commit**

```bash
git add xtask/src/solve.rs xtask/src/main.rs
git commit -m "Integrate test parser into cargo solve with TTY detection"
```

### Task 6: Deprecate `cargo setup`

**Files:**
- Modify: `xtask/src/setup.rs`

- [ ] **Step 1: Replace `setup::run` with deprecation message**

Replace the entire `run` function in `xtask/src/setup.rs`:

```rust
pub fn run() -> Result<(), io::Error> {
    println!("Note: 'cargo setup' has been replaced by './lc setup'.");
    println!("Run ./lc setup from the repo root instead.");
    println!("(./lc works even before Rust/Cargo are installed.)");
    Ok(())
}
```

Remove the `Check` struct, `CHECKS` const, and `check_exists` function — they are no longer used. Keep only `use std::io;` and the `run` function.

- [ ] **Step 2: Run tests**

Run: `cargo test -p xtask`

Expected: all tests PASS

- [ ] **Step 3: Commit**

```bash
git add xtask/src/setup.rs
git commit -m "Deprecate cargo setup in favor of ./lc setup"
```

---

## Chunk 4: Shell Script

### Task 7: Create `./lc`

**Files:**
- Create: `lc` (at repo root, executable)

- [ ] **Step 1: Write the script**

Create `lc` at the repo root:

```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BOLD='\033[1m'
NC='\033[0m'

ALL_GOOD=true

check_tool() {
    local name="$1"
    local required="$2"
    local hint="$3"

    if command -v "$name" &>/dev/null; then
        local version
        version="$("$name" --version 2>/dev/null | head -1)" || version=""
        echo -e "  ${GREEN}✓${NC} ${version:-$name}"
    else
        local label
        if [[ "$required" == "true" ]]; then
            label="required"
            ALL_GOOD=false
        else
            label="recommended"
        fi
        echo -e "  ${RED}✗${NC} ${name} (${label})"
        echo "    → $hint"
    fi
}

cmd_setup() {
    echo "Checking environment..."
    echo ""

    local os
    os="$(uname -s)"
    echo -e "  Platform: ${BOLD}${os}${NC}"
    echo ""

    check_tool "rustup" "true" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    check_tool "rustc" "true" "Install via rustup (see above)"
    check_tool "cargo" "true" "Install via rustup (see above)"
    check_tool "rustfmt" "false" "rustup component add rustfmt"
    check_tool "clippy-driver" "false" "rustup component add clippy"
    check_tool "cargo-watch" "false" "cargo install cargo-watch"

    echo ""

    if [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
        echo -e "${YELLOW}Warning:${NC} ~/.cargo/bin is not in your PATH"
        echo "  Add this to your shell config (~/.bashrc or ~/.zshrc):"
        echo "    export PATH=\"\$HOME/.cargo/bin:\$PATH\""
        echo ""
    fi

    if $ALL_GOOD; then
        echo -e "${GREEN}Ready!${NC} Run 'cargo solve' to start a new problem."
    else
        echo "Some required tools are missing. Install them and run './lc setup' again."
    fi
}

cmd_reset() {
    if ! command -v git &>/dev/null; then
        echo "Error: git is not available. Consider re-cloning the repository."
        exit 1
    fi

    if ! git -C "$SCRIPT_DIR" rev-parse --is-inside-work-tree &>/dev/null 2>&1; then
        echo "Error: not inside a git repository. Consider re-cloning."
        exit 1
    fi

    echo "Restoring source files to clean state..."

    local files=(src/lib.rs src/types.rs src/macros.rs src/solution.rs)
    for file in "${files[@]}"; do
        if git -C "$SCRIPT_DIR" checkout -- "$file" 2>/dev/null; then
            echo "  Restored $file"
        else
            echo -e "  ${YELLOW}Warning:${NC} could not restore $file"
        fi
    done

    if [[ -f "$SCRIPT_DIR/.solve_session" ]]; then
        rm "$SCRIPT_DIR/.solve_session"
        echo "  Removed .solve_session"
    fi

    echo ""
    echo "Done. Your archive/ is untouched."
}

cmd_help() {
    echo -e "${BOLD}LeetCode Rust — Help${NC}"
    echo ""
    echo "  1) How to use this repo"
    echo "  2) Broken something? (reset source files)"
    echo "  3) Missing tools? (check & install)"
    echo "  4) VS Code + rust-analyzer setup"
    echo "  5) AI tutor setup"
    echo "  6) Report a bug"
    echo ""
    read -rp "Pick a number (or press Enter to exit): " choice

    case "$choice" in
        1)
            echo ""
            echo "Read README.md for the full guide:"
            echo "  less \"$SCRIPT_DIR/README.md\""
            ;;
        2)
            echo ""
            cmd_reset
            ;;
        3)
            echo ""
            cmd_setup
            ;;
        4)
            echo ""
            echo "VS Code + rust-analyzer setup:"
            echo "  1. Install the 'rust-analyzer' extension from the VS Code marketplace"
            echo "  2. Open this folder in VS Code: code ."
            echo "  3. rust-analyzer should activate automatically"
            echo "  4. If not, check that rustup is installed: rustup --version"
            echo ""
            echo "Useful settings (add to .vscode/settings.json):"
            echo '  { "rust-analyzer.check.command": "clippy" }'
            ;;
        5)
            echo ""
            if [[ -f "$SCRIPT_DIR/docs/ai-tutor.md" ]]; then
                echo "Read the AI tutor guide:"
                echo "  less \"$SCRIPT_DIR/docs/ai-tutor.md\""
            else
                echo "AI tutor guide not yet available."
            fi
            ;;
        6)
            echo ""
            local remote_url
            remote_url="$(git -C "$SCRIPT_DIR" remote get-url origin 2>/dev/null || echo "")"
            if [[ -n "$remote_url" ]]; then
                local repo_url="${remote_url%.git}"
                repo_url="${repo_url/git@github.com:/https://github.com/}"
                echo "Report issues at:"
                echo "  ${repo_url}/issues"
            else
                echo "Open an issue on the project's GitHub repository."
            fi
            ;;
        *)
            ;;
    esac
}

case "${1:-help}" in
    setup)  cmd_setup ;;
    reset)  cmd_reset ;;
    help)   cmd_help ;;
    *)
        echo "Unknown command: $1"
        echo "Usage: ./lc [setup|reset|help]"
        exit 1
        ;;
esac
```

- [ ] **Step 2: Make executable**

```bash
chmod +x lc
```

- [ ] **Step 3: Manual smoke test**

Run: `./lc setup`

Expected: sees green checkmarks for installed tools, recommendations for missing ones.

Run: `./lc help`

Expected: numbered menu appears, responds to input.

- [ ] **Step 4: Commit**

```bash
git add lc
git commit -m "Add ./lc bootstrap script (setup, reset, help)"
```

---

## Chunk 5: Documentation

### Task 8: Create `docs/fundamentals.md`

**Files:**
- Create: `docs/fundamentals.md`

Note: `docs/cheatsheet.md` already covers patterns in depth (iteration, sorting, data structures, ownership). This file focuses on the "what is this syntax" quick-reference — the moments when 42 students see unfamiliar Rust syntax and want a fast answer. Minimal overlap with the cheatsheet.

- [ ] **Step 1: Write `docs/fundamentals.md`**

```markdown
# Rust Syntax for C Programmers — Quick Reference

Things that look weird the first time you see them. For deeper patterns, see `cheatsheet.md`.

---

## `::` vs `.`

In C, `.` accesses struct members. In Rust, both `::` and `.` exist but serve different roles.

```rust
// :: is the path separator — used for modules, associated functions, enum variants
use std::collections::HashMap;
let m = HashMap::new();          // HashMap::new() is an associated function (like a "static method")
let v = Option::Some(5);         // enum variant

// . calls methods on a value
let len = v.unwrap();
let s = String::from("hello");
let upper = s.to_uppercase();    // method call on the String value
```

**Rule of thumb:** `::` navigates to something (a module, a type, a function). `.` calls something on a value you already have.

---

## `&` and `*` — Not Like C

In C, `&` takes an address and `*` dereferences a pointer. Rust reuses the same symbols but with different semantics.

```c
// C
int x = 5;
int *p = &x;    // p is a raw pointer
*p = 10;        // write through pointer
```

```rust
// Rust
let x = 5;
let r = &x;      // r is a reference (not a raw pointer)
let v = *r;      // dereference to get the value

fn add_one(n: &i32) -> i32 {     // borrows, does not own
    *n + 1
}
```

References in Rust are checked at compile time. They cannot be null, cannot dangle, and cannot alias a mutable reference.

---

## `let` — Immutable by Default

```rust
let x = 5;       // immutable — cannot reassign
let mut y = 5;   // mutable — can reassign
y = 10;

// x = 10;       // compile error
```

In C everything is mutable unless you add `const`. Rust flips the default.

---

## `&[T]` vs `[T; N]` vs `Vec<T>`

```rust
let array: [i32; 3] = [1, 2, 3];     // fixed size, on the stack (like C array)
let vec: Vec<i32> = vec![1, 2, 3];    // growable, on the heap (like malloc'd array)
let slice: &[i32] = &vec[1..3];       // borrowed view into contiguous memory (like pointer + length)
```

Most functions take `&[T]` (slice) because it works with both arrays and Vecs.

---

## `Option<T>` and `Result<T, E>`

There is no `NULL` in Rust. There is no `errno`.

```rust
// Option: value might be absent
let found: Option<i32> = vec.iter().find(|&&x| x == 5).copied();
match found {
    Some(val) => println!("got {val}"),
    None => println!("not found"),
}

// Result: operation might fail
let parsed: Result<i32, _> = "42".parse();
match parsed {
    Ok(n) => println!("parsed {n}"),
    Err(e) => println!("failed: {e}"),
}
```

---

## `impl` Blocks

Methods are defined outside the struct definition, inside `impl` blocks.

```rust
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {    // associated function (no self)
        Point { x, y }
    }

    fn distance(&self) -> f64 {         // method (takes &self)
        ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt()
    }
}

let p = Point::new(3, 4);   // ::new — associated function
let d = p.distance();       // .distance() — method
```

---

## Macros — The `!`

If it ends with `!`, it is a macro, not a function.

```rust
println!("hello");         // macro — can accept format strings
vec![1, 2, 3];             // macro — creates a Vec with initial values
assert_eq!(a, b);          // macro — shows both values on failure
format!("x = {x}");       // macro — returns a String
```

Macros can do things functions cannot: variable argument counts, code generation, compile-time string formatting.

---

## `use`, `mod`, `crate::`

In C you have `#include`. Rust has a module system.

```rust
mod types;                          // declares a module (loads types.rs)
use crate::types::ListNode;        // brings a type into scope
use std::collections::HashMap;     // from the standard library
```

- `crate::` — root of the current project
- `super::` — parent module
- `self::` — current module

---

## `String` vs `&str`

```rust
let owned: String = String::from("hello");   // heap-allocated, owned, growable
let borrowed: &str = "hello";                // string literal, borrowed, immutable
let also_borrowed: &str = &owned;            // borrow a String as &str

fn greet(name: &str) {                       // prefer &str in function params
    println!("hello {name}");
}

greet(&owned);       // works
greet(borrowed);     // works
```

In C, all strings are `char*`. In Rust, `String` owns data, `&str` borrows it.

---

## `std` — The Standard Library

Rust's standard library is at `std::`. Common modules:

```rust
use std::collections::{HashMap, HashSet, VecDeque, BinaryHeap};
use std::cmp::{min, max, Ordering};
use std::rc::Rc;
use std::cell::RefCell;
```

Browse docs at https://doc.rust-lang.org/std/ — press `K` in Neovim on any std type to see its docs.
```

- [ ] **Step 2: Commit**

```bash
git add docs/fundamentals.md
git commit -m "Add C-to-Rust syntax quick reference"
```

### Task 9: Extend `docs/resources.md` with Neetcode Roadmap

**Files:**
- Modify: `docs/resources.md`

- [ ] **Step 1: Append neetcode section**

Add to the end of `docs/resources.md`:

```markdown

---

## LeetCode Problem Roadmap (Neetcode)

Based on the Neetcode roadmap — a structured order for working through LeetCode problems by topic. Each category builds on the previous. Links go to neetcode.io where you can see the full problem lists and video explanations.

### Pick Your Weakness

Not sure where to start? Match your situation:

- **"I struggle with basic data structure operations"** → Start with Arrays & Hashing, then Stack
- **"I can solve easy problems but medium ones feel impossible"** → Two Pointers and Sliding Window — the technique gap between easy and medium
- **"Recursion confuses me"** → Trees, then Backtracking — recursive patterns that build naturally
- **"I don't know when to use which data structure"** → Heap / Priority Queue and Graphs — choosing the right tool
- **"Dynamic programming is black magic"** → work through the DP section last, after all other categories

### Categories

**Arrays & Hashing** — the foundation. Most LeetCode problems touch arrays. HashMap/HashSet for O(1) lookups.
https://neetcode.io/roadmap (Arrays & Hashing section)

**Two Pointers** — scanning from both ends or at different speeds. Turns O(n²) brute force into O(n).
https://neetcode.io/roadmap (Two Pointers section)

**Sliding Window** — fixed or variable-size windows over arrays/strings. Substring and subarray problems.
https://neetcode.io/roadmap (Sliding Window section)

**Stack** — LIFO for matching brackets, monotonic stacks, expression evaluation. Vec::push/pop in Rust.
https://neetcode.io/roadmap (Stack section)

**Binary Search** — not just sorted arrays. Search spaces, boundaries, rotated arrays.
https://neetcode.io/roadmap (Binary Search section)

**Linked List** — ownership gets real here. `Option<Box<ListNode>>`, `.take()`, recursive vs iterative.
https://neetcode.io/roadmap (Linked List section)

**Trees** — `Rc<RefCell<TreeNode>>` everywhere. DFS (pre/in/post-order), BFS, recursive thinking.
https://neetcode.io/roadmap (Trees section)

**Tries** — prefix trees for string problems. Less common but when you need one, nothing else works.
https://neetcode.io/roadmap (Tries section)

**Backtracking** — generate all combinations/permutations. Recursive with state tracking.
https://neetcode.io/roadmap (Backtracking section)

**Heap / Priority Queue** — `BinaryHeap` in Rust (max-heap by default; wrap in `Reverse` for min-heap).
https://neetcode.io/roadmap (Heap / Priority Queue section)

**Graphs** — adjacency lists, DFS, BFS, topological sort, union-find.
https://neetcode.io/roadmap (Graphs section)

**Dynamic Programming** — build from subproblems. Start with 1D DP, then 2D. Hardest category for most people.
https://neetcode.io/roadmap (1-D DP and 2-D DP sections)

**Greedy** — local optimal choices that lead to global optimal. Intervals, scheduling.
https://neetcode.io/roadmap (Greedy section)

**Intervals** — merge, insert, overlap detection. Sort by start time, scan linearly.
https://neetcode.io/roadmap (Intervals section)

**Math & Geometry** — modular arithmetic, GCD, matrix operations.
https://neetcode.io/roadmap (Math & Geometry section)
```

- [ ] **Step 2: Commit**

```bash
git add docs/resources.md
git commit -m "Add neetcode roadmap and weakness guide to resources"
```

### Task 10: Create `docs/ai-tutor.md`

**Files:**
- Create: `docs/ai-tutor.md`

- [ ] **Step 1: Write `docs/ai-tutor.md`**

```markdown
# Using AI as a Rust Tutor

AI coding assistants work well as tutors when given the right context about who you are and what you are learning. This guide covers how to set one up with this repo.

---

## Claude Code

Claude Code is Anthropic's CLI tool for working with code.

### Setup

1. Install: `npm install -g @anthropic-ai/claude-code`
2. Navigate to this repo: `cd path/to/leetcode-rust`
3. Run: `claude`

Claude Code reads the `CLAUDE.md` file in this repo automatically. It already knows this is a learning environment for 42 students transitioning from C to Rust.

### Learning Output Style

Claude Code has a `learning` output style that changes how it interacts with you:

- Provides educational insights about implementation choices
- Asks you to write key pieces of code yourself (design decisions, algorithms)
- Explains Rust concepts in relation to C when relevant

To enable it, add to your Claude Code settings or run with the appropriate flag. Check `claude --help` for current options.

### Example Prompts

When working on a problem:

- "explain this compiler error" — paste the error, get a breakdown
- "walk me through the ownership in this solution" — understand why borrows/moves happen
- "what Rust concept should I practice next based on my archive?" — personalized progression
- "how would I solve this differently in C vs Rust?" — contrastive learning
- "why does the borrow checker reject this?" — understand the rules, not just the fix

### Example Workflow

```
$ cargo solve
# paste your LeetCode examples when prompted
# open src/solution.rs
$ claude
> I'm working on two-sum. Here's my approach: iterate through the array
> and for each element, check if target - element exists in a HashMap.
> Can you help me write this in Rust?
```

---

## Other AI Tools

If you use Cursor, GitHub Copilot, Gemini CLI, or another AI assistant, you can give it the same context by creating a configuration file.

### AGENTS.md

Create a file called `AGENTS.md` in the repo root with this content:

```markdown
# Context

This is a learning environment for 42 school students practicing LeetCode in Rust.

## User Profile
- Strong C programming background (42 school curriculum)
- New to Rust — learning ownership, borrowing, lifetimes, and idiomatic patterns
- Using this repo to solve LeetCode problems and build Rust fluency
- No sudo access on school machines

## How to Help
- Explain Rust concepts in relation to C equivalents
- Highlight common pitfalls for C programmers (mutability defaults, ownership, no NULL)
- Encourage hands-on practice rather than giving complete solutions
- Help debug compiler errors with explanations, not just fixes
- Reference docs/cheatsheet.md and docs/fundamentals.md for patterns

## Project Structure
- src/solution.rs — the working file for the current problem
- src/types.rs — ListNode, TreeNode definitions
- src/macros.rs — list![], tree![] test helpers
- archive/ — solved problems
- docs/ — learning resources
```

### Cursor

Cursor reads `.cursorrules` in the project root. Create one with similar content to the AGENTS.md above.

### GitHub Copilot

Copilot reads `.github/copilot-instructions.md`. Create one with similar content.

---

## Tips for Learning with AI

1. **Ask "why" before "how"** — understanding the reason behind a pattern teaches more than the pattern itself
2. **Try first, then ask** — attempt the problem for 10-15 minutes before asking for help
3. **Request explanations of compiler errors** — Rust's error messages are famously good; AI can break them down further
4. **Ask for alternative approaches** — there is usually more than one way to solve a problem in Rust
5. **Review suggested code critically** — AI is not always right; understanding why code works is more valuable than having working code
```

- [ ] **Step 2: Commit**

```bash
git add docs/ai-tutor.md
git commit -m "Add AI tutor setup guide (Claude Code + other tools)"
```

### Task 11: Update README.md

**Files:**
- Modify: `README.md` (or create if it doesn't exist)

- [ ] **Step 1: Check if README.md exists**

Run: `ls -la README.md`

If it doesn't exist, create it. If it does, read it and update accordingly.

- [ ] **Step 2: Write/update README.md**

The README should contain:

1. One-paragraph description
2. Quick start pointing to `./lc setup`
3. Workflow diagram (text-based)
4. Commands reference table (both `./lc` and `cargo` commands)
5. Project structure
6. Links to docs

```markdown
# LeetCode in Rust

A local workspace for solving LeetCode problems in Rust with full IDE support (rust-analyzer), solution archiving, and progress tracking. Built for 42 school students learning Rust from C.

---

## Quick Start

```bash
git clone <repo-url>
cd leetcode-rust
./lc setup              # checks your environment, tells you what to install
cargo solve             # resets solution.rs, optionally generates tests from examples
# open src/solution.rs, paste your solution
cargo watch -x test     # live test feedback as you code
cargo archive two-sum -d easy -t "array,hash-map" -r "iterators,entry-api"
cargo progress          # see your stats
```

## Commands

| Command | Description |
|---|---|
| `./lc setup` | Check environment, print install commands for missing tools |
| `./lc reset` | Restore source files to clean state (keeps archive) |
| `./lc help` | Interactive help menu |
| `cargo solve` | Start a new problem (reset template, optionally paste LeetCode examples for auto-generated tests) |
| `cargo solve --force` | Overwrite solution.rs without confirmation |
| `cargo archive <name>` | Save current solution to archive/ with metadata |
| `cargo progress` | Show solving stats and progress |
| `cargo watch -x test` | Auto-run tests on file changes |

## Workflow

```
./lc setup → cargo solve → edit solution.rs → cargo watch -x test → cargo archive
                ↑                                                        |
                └────────────────────────────────────────────────────────┘
```

When you run `cargo solve`, you can optionally paste LeetCode examples to auto-generate test cases. Paste the examples (the `Example 1: Input: ... Output: ...` block), press Ctrl+D to confirm, and the tests appear in solution.rs ready to use. Or just press Enter to skip and write tests manually.

## Project Structure

```
leetcode-rust/
├── lc                   # Bootstrap script (setup, reset, help)
├── src/
│   ├── lib.rs           # Crate root
│   ├── solution.rs      # YOUR WORKING FILE
│   ├── types.rs         # ListNode, TreeNode
│   └── macros.rs        # list![], tree![] test helpers
├── xtask/               # CLI tooling (solve, archive, progress)
├── archive/             # Your solved problems
└── docs/
    ├── fundamentals.md  # Rust syntax quick reference (for C programmers)
    ├── cheatsheet.md    # C-to-Rust patterns in depth
    ├── resources.md     # Learning resources + neetcode roadmap
    └── ai-tutor.md      # Setting up AI assistance
```

## Documentation

- **[Fundamentals](docs/fundamentals.md)** — "What does `::` mean?" Quick answers to Rust syntax that surprises C programmers
- **[Cheatsheet](docs/cheatsheet.md)** — Side-by-side C and Rust patterns for LeetCode
- **[Resources](docs/resources.md)** — Curated learning resources + neetcode problem roadmap
- **[AI Tutor](docs/ai-tutor.md)** — Set up Claude Code or other AI tools as your Rust tutor

## Requirements

- macOS or Linux
- Rust toolchain (installed via [rustup](https://rustup.rs/))
- Recommended: `cargo-watch` for live test feedback
```

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "Add README with quick start, commands, and doc links"
```

- [ ] **Step 4: Run full test suite**

Run: `cargo test`

Expected: all tests PASS across all crates.
