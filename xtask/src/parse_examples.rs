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
            .filter(|e| !e.trim().is_empty())
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

        let eq_pos = segment.find('=')?;

        let name = segment[..eq_pos].trim();
        let value = segment[eq_pos + 1..].trim();

        let rust_value = parse_value(value)?;
        params.push((name.to_string(), rust_value));
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
            let rest = s[i + 1..].trim_start();
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

pub struct Example {
    pub params: Vec<(String, String)>,
    pub output: String,
}

pub struct ParseResult {
    pub examples: Vec<Example>,
    pub warnings: Vec<String>,
}

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

pub fn generate_test_code(result: &ParseResult, method_name: Option<&str>) -> String {
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

            match method_name {
                Some(name) => {
                    code.push_str(&format!(
                        "        let result = Solution::{name}({args});\n"
                    ));
                    code.push_str("        assert_eq!(result, expected);\n");
                }
                None => {
                    code.push_str(
                        "        // TODO: uncomment and replace method_name with your method\n"
                    );
                    code.push_str(&format!(
                        "        // let result = Solution::method_name({args});\n"
                    ));
                    code.push_str("        // assert_eq!(result, expected);\n");
                }
            }
            code.push_str("    }\n");
        }
    }

    code.push_str("}\n");
    code
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
        let code = generate_test_code(&result, None);

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
        let code = generate_test_code(&result, None);

        assert!(code.contains("fn example_1()"));
        assert!(code.contains("fn example_2()"));
    }

    #[test]
    fn generate_empty_gives_default() {
        let result = parse_examples("");
        let code = generate_test_code(&result, None);

        assert!(code.contains("fn example()"));
        assert!(code.contains("// your tests here"));
    }

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
        let code = generate_test_code(&result, None);
        assert!(code.contains("fn example()"));
    }

    #[test]
    fn generate_with_method_name() {
        let input = "\
Example 1:

Input: nums = [2,7,11,15], target = 9
Output: [0,1]";

        let result = parse_examples(input);
        let code = generate_test_code(&result, Some("two_sum"));

        assert!(code.contains("fn example_1()"));
        assert!(code.contains("let nums = vec![2, 7, 11, 15];"));
        assert!(code.contains("let target = 9;"));
        assert!(code.contains("let expected = vec![0, 1];"));
        assert!(code.contains("let result = Solution::two_sum(nums, target);"));
        assert!(code.contains("assert_eq!(result, expected);"));
        assert!(!code.contains("// TODO"));
    }

    #[test]
    fn generate_without_method_name() {
        let input = "\
Example 1:

Input: x = 121
Output: true";

        let result = parse_examples(input);
        let code = generate_test_code(&result, None);

        assert!(code.contains("// TODO"));
        assert!(code.contains("// let result = Solution::method_name(x);"));
    }

    #[test]
    fn generate_empty_with_method_name() {
        let result = parse_examples("");
        let code = generate_test_code(&result, Some("two_sum"));

        assert!(code.contains("fn example()"));
        assert!(code.contains("// your tests here"));
    }
}
