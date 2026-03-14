pub fn extract_slug(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    let url = url.split('?').next().unwrap_or(url);
    let url = url.split('#').next().unwrap_or(url);

    let url = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;

    if !url.starts_with("leetcode.com/") {
        return None;
    }

    let path = url.strip_prefix("leetcode.com")?;
    let path = path.strip_prefix("/problems/")?;

    let slug = path.split('/').next().unwrap_or(path);
    let slug = slug.trim();

    if slug.is_empty() {
        return None;
    }

    Some(slug.to_string())
}

pub fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut tag_name = String::new();
    let chars: Vec<char> = html.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '<' {
            in_tag = true;
            tag_name.clear();
            i += 1;
            continue;
        }

        if in_tag {
            if chars[i] == '>' {
                in_tag = false;
                let tag = tag_name.to_lowercase();
                let tag_base = tag.split_whitespace().next().unwrap_or("");
                let tag_base = tag_base.trim_start_matches('/');

                let is_closing = tag.starts_with('/');
                match tag_base {
                    "p" | "pre" | "div" if is_closing => {
                        result.push('\n');
                    }
                    "p" | "pre" | "div" if !is_closing => {
                        if !result.is_empty() && !result.ends_with('\n') {
                            result.push('\n');
                        }
                    }
                    "li" if is_closing => {
                        if !result.ends_with('\n') {
                            result.push('\n');
                        }
                    }
                    "br" | "br/" => {
                        result.push('\n');
                    }
                    "sup" => {
                        if !tag.starts_with('/') {
                            result.push('^');
                        }
                    }
                    _ => {}
                }
                i += 1;
            } else {
                tag_name.push(chars[i]);
                i += 1;
            }
            continue;
        }

        if chars[i] == '&' {
            let rest: String = chars[i..].iter().take(12).collect();
            if rest.starts_with("&amp;") {
                result.push('&');
                i += 5;
            } else if rest.starts_with("&lt;") {
                result.push('<');
                i += 4;
            } else if rest.starts_with("&gt;") {
                result.push('>');
                i += 4;
            } else if rest.starts_with("&nbsp;") {
                result.push(' ');
                i += 6;
            } else if rest.starts_with("&quot;") {
                result.push('"');
                i += 6;
            } else if rest.starts_with("&#39;") {
                result.push('\'');
                i += 5;
            } else if rest.starts_with("&#x") || rest.starts_with("&#X") {
                if let Some(semi) = rest.find(';') {
                    let hex = &rest[3..semi];
                    if let Ok(code) = u32::from_str_radix(hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        }
                    }
                    i += semi + 1;
                } else {
                    result.push('&');
                    i += 1;
                }
            } else if rest.starts_with("&#") {
                if let Some(semi) = rest.find(';') {
                    let num = &rest[2..semi];
                    if let Ok(code) = num.parse::<u32>() {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        }
                    }
                    i += semi + 1;
                } else {
                    result.push('&');
                    i += 1;
                }
            } else {
                result.push('&');
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_basic_url() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/two-sum/"),
            Some("two-sum".into())
        );
    }

    #[test]
    fn slug_no_trailing_slash() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/two-sum"),
            Some("two-sum".into())
        );
    }

    #[test]
    fn slug_with_description_suffix() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/two-sum/description/"),
            Some("two-sum".into())
        );
    }

    #[test]
    fn slug_with_query_params() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/two-sum/?envType=daily-question&envId=2024-01-01"),
            Some("two-sum".into())
        );
    }

    #[test]
    fn slug_with_fragment() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/two-sum/#description"),
            Some("two-sum".into())
        );
    }

    #[test]
    fn slug_http_url() {
        assert_eq!(
            extract_slug("http://leetcode.com/problems/two-sum/"),
            Some("two-sum".into())
        );
    }

    #[test]
    fn slug_not_leetcode() {
        assert_eq!(extract_slug("https://example.com/problems/two-sum/"), None);
    }

    #[test]
    fn slug_missing_problems_path() {
        assert_eq!(extract_slug("https://leetcode.com/contest/weekly/"), None);
    }

    #[test]
    fn slug_empty() {
        assert_eq!(extract_slug(""), None);
    }

    #[test]
    fn slug_garbage() {
        assert_eq!(extract_slug("not a url at all"), None);
    }

    #[test]
    fn slug_just_domain() {
        assert_eq!(extract_slug("https://leetcode.com/"), None);
    }

    #[test]
    fn slug_problems_but_no_slug() {
        assert_eq!(extract_slug("https://leetcode.com/problems/"), None);
    }

    #[test]
    fn slug_with_description_and_query() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/valid-parentheses/description/?envType=study-plan"),
            Some("valid-parentheses".into())
        );
    }

    #[test]
    fn slug_complex_name() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/longest-substring-without-repeating-characters/"),
            Some("longest-substring-without-repeating-characters".into())
        );
    }

    #[test]
    fn slug_numeric_name() {
        assert_eq!(
            extract_slug("https://leetcode.com/problems/3sum/"),
            Some("3sum".into())
        );
    }

    #[test]
    fn strip_simple_tags() {
        assert_eq!(strip_html("<p>hello</p>"), "hello\n");
    }

    #[test]
    fn strip_nested_tags() {
        assert_eq!(
            strip_html("<p><strong>bold</strong> text</p>"),
            "bold text\n"
        );
    }

    #[test]
    fn strip_pre_block() {
        assert_eq!(
            strip_html("<pre>\nline1\nline2\n</pre>"),
            "\nline1\nline2\n\n"
        );
    }

    #[test]
    fn strip_entities() {
        assert_eq!(strip_html("a &amp; b &lt; c &gt; d"), "a & b < c > d");
    }

    #[test]
    fn strip_nbsp() {
        assert_eq!(strip_html("a&nbsp;b"), "a b");
    }

    #[test]
    fn strip_quot_entity() {
        assert_eq!(strip_html("&quot;hello&quot;"), "\"hello\"");
    }

    #[test]
    fn strip_numeric_entity() {
        assert_eq!(strip_html("&#39;quote&#39;"), "'quote'");
    }

    #[test]
    fn strip_hex_entity() {
        assert_eq!(strip_html("&#x27;quote&#x27;"), "'quote'");
    }

    #[test]
    fn strip_empty() {
        assert_eq!(strip_html(""), "");
    }

    #[test]
    fn strip_no_tags() {
        assert_eq!(strip_html("plain text"), "plain text");
    }

    #[test]
    fn strip_leetcode_example() {
        let html = r#"<p><strong class="example">Example 1:</strong></p>

<pre>
<strong>Input:</strong> nums = [2,7,11,15], target = 9
<strong>Output:</strong> [0,1]
<strong>Explanation:</strong> Because nums[0] + nums[1] == 9, we return [0, 1].
</pre>"#;

        let result = strip_html(html);
        assert!(result.contains("Example 1:"));
        assert!(result.contains("Input: nums = [2,7,11,15], target = 9"));
        assert!(result.contains("Output: [0,1]"));
    }

    #[test]
    fn strip_list_items() {
        assert_eq!(
            strip_html("<ul><li>item one</li><li>item two</li></ul>"),
            "item one\nitem two\n"
        );
    }

    #[test]
    fn strip_br_tag() {
        assert_eq!(strip_html("line1<br>line2<br/>line3"), "line1\nline2\nline3");
    }

    #[test]
    fn strip_sup_tag() {
        assert_eq!(strip_html("10<sup>4</sup>"), "10^4");
    }
}
