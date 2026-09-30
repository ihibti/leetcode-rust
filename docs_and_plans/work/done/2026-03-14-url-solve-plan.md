# URL-Based Solve — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the paste-based `cargo solve` flow with URL-based LeetCode problem fetching via GraphQL API, and simplify the `./lc` script.

**Architecture:** New `leetcode.rs` module handles URL parsing, HTTP fetching, and HTML stripping — all pure functions except the single `fetch_problem` I/O boundary. `solve.rs` gains a URL parameter and assembles templates using the existing `parse_examples` pipeline. `generate_test_code` gains an optional method name to produce real assertions instead of `// TODO` placeholders.

**Tech Stack:** Rust 2024 edition, `ureq` 3.x for HTTP, standard library only otherwise.

**Spec:** `docs/plans/2026-03-14-url-solve-design.md`

---

## Chunk 1: LeetCode Module — Pure Functions

New file `xtask/src/leetcode.rs`. This chunk covers the pure functions: URL slug extraction, HTML stripping, and method name extraction. No HTTP yet.

### Task 1: URL Slug Extraction

**Files:**
- Create: `xtask/src/leetcode.rs`
- Modify: `xtask/src/main.rs` (add `mod leetcode;`)

- [ ] **Step 1: Register the module**

Add to `xtask/src/main.rs` after line 4 (`mod parse_examples;`):

```rust
mod leetcode;
```

- [ ] **Step 2: Write failing tests for slug extraction**

Create `xtask/src/leetcode.rs` with a stub and tests:

```rust
pub fn extract_slug(url: &str) -> Option<String> {
    None
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
}
```

- [ ] **Step 3: Run tests — verify they fail**

Run: `cargo test -p xtask leetcode::tests::slug_`

Expected: all tests FAIL except `slug_empty` and `slug_garbage` (both expect `None`)

- [ ] **Step 4: Implement `extract_slug`**

Replace the stub:

```rust
pub fn extract_slug(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    let url = url.split('?').next().unwrap_or(url);
    let url = url.split('#').next().unwrap_or(url);

    let url = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://"))?;

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
```

- [ ] **Step 5: Run tests — verify they pass**

Run: `cargo test -p xtask leetcode::tests::slug_`

Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git add xtask/src/leetcode.rs xtask/src/main.rs
git commit -m "Add leetcode module with URL slug extraction"
```

---

### Task 2: HTML Stripping

**Files:**
- Modify: `xtask/src/leetcode.rs`

- [ ] **Step 1: Write failing tests for HTML stripping**

Append to the `tests` module in `xtask/src/leetcode.rs`:

```rust
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
```

- [ ] **Step 2: Add `strip_html` stub**

Add above the test module in `xtask/src/leetcode.rs`:

```rust
pub fn strip_html(html: &str) -> String {
    String::new()
}
```

- [ ] **Step 3: Run tests — verify new tests fail**

Run: `cargo test -p xtask leetcode::tests::strip_`

Expected: all `strip_*` tests FAIL

- [ ] **Step 4: Implement `strip_html`**

Replace the stub:

```rust
pub fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut capturing_tag = false;
    let chars: Vec<char> = html.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if chars[i] == '<' {
            in_tag = true;
            tag_name.clear();
            capturing_tag = true;
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
                        if !result.ends_with('\n') {
                            result.push('\n');
                        }
                    }
                    "p" | "pre" | "div" if !is_closing => {
                        if !result.is_empty() && !result.ends_with('\n') {
                            result.push('\n');
                        }
                    }
                    "li" => {
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
                if capturing_tag {
                    tag_name.push(chars[i]);
                }
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
```

- [ ] **Step 5: Run tests — verify they pass**

Run: `cargo test -p xtask leetcode::tests::strip_`

Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git add xtask/src/leetcode.rs
git commit -m "Add HTML stripping with entity decoding"
```

---

### Task 3: Method Name Extraction

**Files:**
- Modify: `xtask/src/leetcode.rs`

- [ ] **Step 1: Write failing tests for method name extraction**

Append to the `tests` module:

```rust
    #[test]
    fn method_basic() {
        assert_eq!(
            extract_method_name("impl Solution {\n    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {\n        \n    }\n}"),
            Some("two_sum".into())
        );
    }

    #[test]
    fn method_single_param() {
        assert_eq!(
            extract_method_name("impl Solution {\n    pub fn is_palindrome(x: i32) -> bool {\n        \n    }\n}"),
            Some("is_palindrome".into())
        );
    }

    #[test]
    fn method_with_self() {
        assert_eq!(
            extract_method_name("impl Solution {\n    pub fn run(&self) -> i32 {\n        \n    }\n}"),
            Some("run".into())
        );
    }

    #[test]
    fn method_empty_snippet() {
        assert_eq!(extract_method_name(""), None);
    }

    #[test]
    fn method_no_pub_fn() {
        assert_eq!(extract_method_name("struct Foo {}"), None);
    }

    #[test]
    fn method_complex_return_type() {
        assert_eq!(
            extract_method_name("impl Solution {\n    pub fn max_area(height: Vec<i32>) -> i32 {\n        \n    }\n}"),
            Some("max_area".into())
        );
    }

    #[test]
    fn method_lifetime_annotation() {
        assert_eq!(
            extract_method_name("impl Solution {\n    pub fn longest_common_prefix<'a>(strs: &[&'a str]) -> &'a str {\n        \n    }\n}"),
            Some("longest_common_prefix".into())
        );
    }
```

- [ ] **Step 2: Add `extract_method_name` stub**

Add above the test module:

```rust
pub fn extract_method_name(rust_snippet: &str) -> Option<String> {
    None
}
```

- [ ] **Step 3: Run tests — verify new tests fail**

Run: `cargo test -p xtask leetcode::tests::method_`

Expected: all `method_*` tests FAIL except `method_empty_snippet` and `method_no_pub_fn`

- [ ] **Step 4: Implement `extract_method_name`**

Replace the stub:

```rust
pub fn extract_method_name(rust_snippet: &str) -> Option<String> {
    for line in rust_snippet.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub fn ") {
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}
```

- [ ] **Step 5: Run tests — verify they pass**

Run: `cargo test -p xtask leetcode::tests::method_`

Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git add xtask/src/leetcode.rs
git commit -m "Add method name extraction from Rust snippets"
```

---

## Chunk 2: HTTP Fetching & Error Types

### Task 4: Fetch Problem from GraphQL API

**Files:**
- Modify: `xtask/src/leetcode.rs`
- Modify: `xtask/Cargo.toml` (add `ureq` dependency)

- [ ] **Step 1: Add `ureq` dependency**

Add to `xtask/Cargo.toml` under `[dependencies]`:

```toml
ureq = "3"
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build -p xtask`

Expected: compiles successfully (downloads ureq and dependencies)

- [ ] **Step 3: Add error type and `ProblemData` struct**

Add at the top of `xtask/src/leetcode.rs` (after any existing `use` statements):

```rust
use std::fmt;
use std::io::Read;

pub struct ProblemData {
    pub slug: String,
    pub title: String,
    pub examples_text: String,
    pub rust_snippet: String,
}

pub enum FetchError {
    Network(String),
    InvalidResponse(String),
    NoRustSnippet,
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::Network(msg) => write!(f, "Network error: {msg}"),
            FetchError::InvalidResponse(msg) => write!(f, "Invalid response: {msg}"),
            FetchError::NoRustSnippet => write!(f, "No Rust code snippet found for this problem"),
        }
    }
}
```

- [ ] **Step 4: Implement `fetch_problem`**

Add above the test module:

```rust
pub fn fetch_problem(slug: &str) -> Result<ProblemData, FetchError> {
    let query = r#"{"query":"query getQuestion($titleSlug: String!) { question(titleSlug: $titleSlug) { title content codeSnippets { langSlug code } } }","variables":{"titleSlug":"SLUG"}}"#;
    let body = query.replace("SLUG", slug);

    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(10)))
        .user_agent("leetcode-rust-workspace/0.1")
        .build()
        .new_agent();

    let mut response = agent.post("https://leetcode.com/graphql")
        .header("Content-Type", "application/json")
        .send(body.as_bytes())
        .map_err(|e| FetchError::Network(e.to_string()))?;

    let response_body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| FetchError::Network(e.to_string()))?;

    let title = extract_json_string(&response_body, "title")
        .ok_or_else(|| FetchError::InvalidResponse("missing title".into()))?;

    let content = extract_json_string(&response_body, "content")
        .ok_or_else(|| FetchError::InvalidResponse("missing content (premium problem?)".into()))?;

    let rust_snippet = extract_rust_snippet(&response_body)
        .ok_or(FetchError::NoRustSnippet)?;

    let examples_text = strip_html(&content);

    Ok(ProblemData {
        slug: slug.to_string(),
        title,
        examples_text,
        rust_snippet,
    })
}

fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = &json[start..];
    let mut result = String::new();
    let mut chars = rest.chars();
    loop {
        match chars.next()? {
            '\\' => match chars.next()? {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                'n' => result.push('\n'),
                't' => result.push('\t'),
                '/' => result.push('/'),
                other => {
                    result.push('\\');
                    result.push(other);
                }
            },
            '"' => break,
            c => result.push(c),
        }
    }
    Some(result)
}

fn extract_rust_snippet(json: &str) -> Option<String> {
    let marker = "\"langSlug\":\"rust\"";
    let lang_pos = json.find(marker)?;

    let code_pattern = "\"code\":\"";
    let code_pos = json[lang_pos..].find(code_pattern)?;
    let code_start = lang_pos + code_pos + code_pattern.len();
    let rest = &json[code_start..];

    let mut result = String::new();
    let mut chars = rest.chars();
    loop {
        match chars.next()? {
            '\\' => match chars.next()? {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                'n' => result.push('\n'),
                't' => result.push('\t'),
                '/' => result.push('/'),
                other => {
                    result.push('\\');
                    result.push(other);
                }
            },
            '"' => break,
            c => result.push(c),
        }
    }

    if result.is_empty() { None } else { Some(result) }
}
```

- [ ] **Step 5: Add tests for JSON helpers**

Append to the `tests` module:

```rust
    #[test]
    fn json_extract_title() {
        let json = r#"{"data":{"question":{"title":"Two Sum","content":"<p>desc</p>"}}}"#;
        assert_eq!(extract_json_string(json, "title"), Some("Two Sum".into()));
    }

    #[test]
    fn json_extract_with_escapes() {
        let json = r#"{"data":{"question":{"title":"Contains \"quotes\"","content":"text"}}}"#;
        assert_eq!(
            extract_json_string(json, "title"),
            Some("Contains \"quotes\"".into())
        );
    }

    #[test]
    fn json_extract_missing_key() {
        let json = r#"{"data":{}}"#;
        assert_eq!(extract_json_string(json, "title"), None);
    }

    #[test]
    fn json_extract_content_with_newlines() {
        let json = r#"{"content":"line1\nline2"}"#;
        assert_eq!(
            extract_json_string(json, "content"),
            Some("line1\nline2".into())
        );
    }

    #[test]
    fn rust_snippet_extraction() {
        let json = r#"{"codeSnippets":[{"lang":"C++","langSlug":"cpp","code":"class Solution {}"},{"lang":"Rust","langSlug":"rust","code":"impl Solution {\n    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {\n        \n    }\n}"}]}"#;
        let snippet = extract_rust_snippet(json).unwrap();
        assert!(snippet.contains("pub fn two_sum"));
        assert!(snippet.contains("impl Solution"));
    }

    #[test]
    fn rust_snippet_code_after_lang() {
        let json = r#"{"langSlug":"rust","code":"impl Solution {\n    pub fn foo() {}\n}"}"#;
        let snippet = extract_rust_snippet(json).unwrap();
        assert!(snippet.contains("pub fn foo"));
    }

    #[test]
    fn rust_snippet_missing() {
        let json = r#"{"codeSnippets":[{"lang":"C++","langSlug":"cpp","code":"class Solution {}"}]}"#;
        assert_eq!(extract_rust_snippet(json), None);
    }

    #[test]
    fn rust_snippet_empty_code() {
        let json = r#"{"langSlug":"rust","code":""}"#;
        assert_eq!(extract_rust_snippet(json), None);
    }
```

- [ ] **Step 6: Run all leetcode tests**

Run: `cargo test -p xtask leetcode`

Expected: all tests PASS

- [ ] **Step 7: Commit**

```bash
git add xtask/Cargo.toml xtask/src/leetcode.rs
git commit -m "Add LeetCode GraphQL fetcher with ureq"
```

---

## Chunk 3: Update `generate_test_code` and Integrate into `solve`

### Task 5: Add Method Name Support to `generate_test_code`

**Files:**
- Modify: `xtask/src/parse_examples.rs`

- [ ] **Step 1: Write failing tests for method name parameter**

Append to the `tests` module in `xtask/src/parse_examples.rs`:

```rust
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
```

- [ ] **Step 2: Run tests — verify they fail**

Run: `cargo test -p xtask parse_examples::tests::generate_with`

Expected: FAIL — `generate_test_code` doesn't accept a second parameter yet

- [ ] **Step 3: Update `generate_test_code` signature and implementation**

In `xtask/src/parse_examples.rs`, replace the `generate_test_code` function (line 255):

```rust
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
```

- [ ] **Step 4: Update all existing callers to pass `None`**

In `xtask/src/solve.rs`, line 51, change:

```rust
                Some(crate::parse_examples::generate_test_code(&result))
```

to:

```rust
                Some(crate::parse_examples::generate_test_code(&result, None))
```

In `xtask/src/parse_examples.rs`, update the existing test calls. Find every call to `generate_test_code(&result)` in the test module and add `, None`:

- `generate_basic_test_code`: `generate_test_code(&result)` → `generate_test_code(&result, None)`
- `generate_multiple_tests`: `generate_test_code(&result)` → `generate_test_code(&result, None)`
- `generate_empty_gives_default`: `generate_test_code(&result)` → `generate_test_code(&result, None)`
- `generate_preserves_warnings`: `generate_test_code(&result)` → `generate_test_code(&result, None)`

- [ ] **Step 5: Run all tests**

Run: `cargo test -p xtask`

Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git add xtask/src/parse_examples.rs xtask/src/solve.rs
git commit -m "Add optional method name parameter to generate_test_code"
```

---

### Task 6: Rewire `solve::run` to Accept URL

**Note:** Line numbers referenced below are from the *original* file state before Tasks 1-5. After prior tasks modify these files, locate edit sites by searching for the code patterns shown rather than relying on line numbers.

**Files:**
- Modify: `xtask/src/solve.rs`
- Modify: `xtask/src/main.rs`

- [ ] **Step 1: Update existing tests in `solve.rs`**

All existing tests that pass `example_input: Option<&str>` as the third argument need to change to `url: Option<&str>` — but since most pass `None`, only the names change conceptually. The tests that pass `Some(examples)` (raw text) will be removed and replaced.

In `xtask/src/solve.rs`, remove these two tests entirely:
- `solve_with_valid_examples` (line 116)
- `solve_with_garbage_examples` (line 138)

Keep `solve_with_none_gives_clean_template` — rename to `solve_no_url_gives_template`.

- [ ] **Step 2: Add new integration tests**

Replace the removed tests with:

```rust
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

    // Note: no integration test for the URL success path to avoid network dependency.
    // The success path is tested via assemble_template unit tests below and manual E2E in Task 9.
```

- [ ] **Step 3: Add a template assembly helper and its tests**

Add above the test module in `xtask/src/solve.rs`:

```rust
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
```

Add tests for it in the test module:

```rust
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
```

- [ ] **Step 4: Rewrite `solve::run`**

Replace the entire `run` function in `xtask/src/solve.rs`:

```rust
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
```

- [ ] **Step 5: Update `main.rs` — CLI and dispatch**

In `xtask/src/main.rs`, update the `Solve` variant (line 22-25):

```rust
    #[command(about = "Start a new LeetCode problem")]
    Solve {
        #[arg(long, help = "Overwrite without confirmation")]
        force: bool,
        #[arg(help = "LeetCode problem URL")]
        url: Option<String>,
    },
```

Replace the entire `Command::Solve` arm in `main()` (lines 54-79):

```rust
        Command::Solve { force, url } => solve::run(&root, force, url.as_deref()),
```

- [ ] **Step 6: Run all tests**

Run: `cargo test -p xtask`

Expected: all tests PASS

- [ ] **Step 7: Commit**

```bash
git add xtask/src/solve.rs xtask/src/main.rs
git commit -m "Rewire cargo solve to accept LeetCode URL"
```

---

## Chunk 4: Simplify `./lc` and Update Docs

### Task 7: Strip `./lc` to Setup + Reset

**Files:**
- Modify: `lc`

- [ ] **Step 1: Remove `cmd_help` and update dispatch**

Replace the entire `case` block at the bottom of `lc` (lines 163-172):

```bash
case "${1:-setup}" in
    setup)  cmd_setup ;;
    reset)  cmd_reset ;;
    *)
        echo "Usage: ./lc [setup|reset]"
        exit 1
        ;;
esac
```

Remove the `cmd_help` function entirely (lines 99-161).

- [ ] **Step 2: Smoke test**

Run: `./lc setup`

Expected: environment check output

Run: `./lc`

Expected: same (default is now `setup`)

Run: `./lc help`

Expected: "Usage: ./lc [setup|reset]"

- [ ] **Step 3: Commit**

```bash
git add lc
git commit -m "Simplify ./lc to setup and reset only"
```

### Task 8: Update README and Commands Table

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Update Quick Start section**

Replace the Quick Start code block:

```bash
git clone https://github.com/ihibti/leetcode-rust.git
cd leetcode-rust
./lc setup                                              # checks your environment
cargo solve https://leetcode.com/problems/two-sum/      # fetches problem, generates tests
# open src/solution.rs — impl block and tests are ready
cargo watch -x test                                     # live test feedback as you code
cargo archive two-sum -d easy -t "array,hash-map" -r "iterators,entry-api"
cargo progress                                          # see your stats
```

- [ ] **Step 2: Update Commands table**

Replace the Commands table:

```markdown
| Command | Description |
|---|---|
| `./lc setup` | Check environment, print install commands for missing tools |
| `./lc reset` | Restore source files to clean state (keeps archive) |
| `cargo solve <url>` | Fetch problem from LeetCode, generate impl skeleton and tests |
| `cargo solve` | Start with a blank template (no URL) |
| `cargo solve --force` | Overwrite solution.rs without confirmation |
| `cargo archive <name>` | Save current solution to archive/ with metadata |
| `cargo progress` | Show solving stats and progress |
| `cargo watch -x test` | Auto-run tests on file changes |
```

- [ ] **Step 3: Update Workflow section**

Replace the Workflow text block:

```markdown
## Workflow

```
./lc setup → cargo solve <url> → edit solution.rs → cargo watch -x test → cargo archive
                  ↑                                                            |
                  └────────────────────────────────────────────────────────────┘
```

When you run `cargo solve <url>`, it fetches the problem from LeetCode, generates the `impl Solution` skeleton and test cases from the examples. Open `src/solution.rs` — the method signature and tests are ready, just fill in the implementation.

Running `cargo solve` without a URL gives a blank template.
```

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "Update README for URL-based solve workflow"
```

### Task 9: Manual End-to-End Test

- [ ] **Step 1: Test with a real LeetCode URL**

Run: `cargo solve https://leetcode.com/problems/two-sum/`

Expected output:
```
Fetching problem: two-sum...
Got it: Two Sum
Ready! Open src/solution.rs
```

Check `src/solution.rs` contains:
- `impl Solution { pub fn two_sum(...) }`
- `fn example_1()` with `let nums = vec![2, 7, 11, 15];`
- `let result = Solution::two_sum(nums, target);`

- [ ] **Step 2: Test with no URL**

Run: `cargo solve --force`

Expected: clean template + tip message

- [ ] **Step 3: Test with invalid URL**

Run: `cargo solve --force https://example.com/foo`

Expected: warning + blank template

- [ ] **Step 4: Run full test suite**

Run: `cargo test && cargo test -p xtask`

Expected: all tests PASS

- [ ] **Step 5: Reset solution.rs**

Run: `cargo solve --force`

This restores the clean template so the repo is ready for next use.

- [ ] **Step 6: Commit the plan file**

```bash
git add docs/plans/2026-03-14-url-solve-design.md
git commit -m "Add design spec for URL-based solve"
```
