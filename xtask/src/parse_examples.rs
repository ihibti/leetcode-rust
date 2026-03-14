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
}
