# URL-Based Solve & Script Simplification — Design Spec

## Goal

Replace the paste-based `cargo solve` flow with URL-based LeetCode fetching, and simplify `./lc` by removing the interactive help menu.

## Changes

### 1. New Module: `xtask/src/leetcode.rs`

Responsible for fetching problem data from LeetCode's public GraphQL API.

**Public interface:**

```rust
pub struct ProblemData {
    pub slug: String,
    pub title: String,
    pub examples_text: String,
    pub rust_snippet: String,
}

pub fn extract_slug(url: &str) -> Option<String>;
pub fn fetch_problem(slug: &str) -> Result<ProblemData, FetchError>;
pub fn strip_html(html: &str) -> String;
pub fn extract_method_name(rust_snippet: &str) -> Option<String>;
```

**`extract_slug`**: Validates the input is a LeetCode problem URL. Accepted formats:
- `https://leetcode.com/problems/<slug>/`
- `https://leetcode.com/problems/<slug>`
- `https://leetcode.com/problems/<slug>/description/`
- Any of the above with query parameters (`?envType=...`) or fragments (`#...`) — these are stripped before extraction.

Returns the slug or `None` if not a valid LeetCode URL.

**`fetch_problem`**: POST to `https://leetcode.com/graphql` with query:
```graphql
query getQuestion($titleSlug: String!) {
  question(titleSlug: $titleSlug) {
    title
    content
    codeSnippets { langSlug code }
  }
}
```
Extracts the Rust snippet from `codeSnippets` (filter by `langSlug == "rust"`). Strips HTML from `content` to produce plain text examples. Returns `ProblemData`.

HTTP settings: 10-second timeout, `User-Agent: leetcode-rust-workspace/0.1`.

**`strip_html`**: Removes HTML tags, decodes common entities (`&amp;` → `&`, `&lt;` → `<`, `&gt;` → `>`, `&nbsp;` → ` `, `&quot;` → `"`, `&#39;` → `'`, plus numeric entities). Preserves newlines around block elements (`<p>`, `<pre>`, `<li>`).

Concrete example — LeetCode's actual API response for two-sum contains:
```html
<p><strong class="example">Example 1:</strong></p>\n\n<pre>\n<strong>Input:</strong> nums = [2,7,11,15], target = 9\n<strong>Output:</strong> [0,1]\n<strong>Explanation:</strong> Because nums[0] + nums[1] == 9, we return [0, 1].\n</pre>
```

After `strip_html`, this becomes:
```
Example 1:

Input: nums = [2,7,11,15], target = 9
Output: [0,1]
Explanation: Because nums[0] + nums[1] == 9, we return [0, 1].
```

This matches the format the existing `parse_examples` module expects.

**`extract_method_name`**: Extracts the method name from the Rust snippet using a simple pattern match for `pub fn (\w+)`. Returns `None` if extraction fails (unexpected snippet format).

**Error type:**
```rust
pub enum FetchError {
    Network(String),
    InvalidResponse(String),
    NoRustSnippet,
}
```

`FetchError` is NOT propagated out of `solve::run`. The `run` function matches on it internally, prints a warning, and falls back to the clean template. The `run` signature stays `Result<(), io::Error>`.

### 2. Changes to `solve.rs`

**New signature:**
```rust
pub fn run(root: &Path, force: bool, url: Option<&str>) -> Result<(), io::Error>
```

When `url` is `Some`:
1. Call `extract_slug(url)` — if `None`, print warning "not a valid LeetCode URL", fall back to clean template
2. Call `fetch_problem(slug)` — on error, print warning with error details, fall back to clean template
3. On success: feed `examples_text` to existing `parse_examples` → `generate_test_code(result, method_name)`, combine with `rust_snippet` into solution template

When `url` is `None`:
- Write clean `SOLUTION_TEMPLATE` (current behavior)
- Print tip: `Tip: pass a LeetCode URL to auto-generate tests (cargo solve <url>)`

**Generated template format (when URL fetch succeeds):**
```rust
use crate::types::*;

pub struct Solution;

impl Solution {
    pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{list, tree};

    #[test]
    fn example_1() {
        let nums = vec![2, 7, 11, 15];
        let target = 9;
        let expected = vec![0, 1];
        let result = Solution::two_sum(nums, target);
        assert_eq!(result, expected);
    }
}
```

The `rust_snippet` from LeetCode is pasted verbatim (it is always just the `impl` block).

### 3. Changes to `main.rs`

Update CLI:
```rust
Solve {
    #[arg(long, help = "Overwrite without confirmation")]
    force: bool,
    #[arg(help = "LeetCode problem URL")]
    url: Option<String>,
}
```

Remove TTY detection and stdin reading. Pass `url.as_deref()` to `solve::run`.

### 4. Changes to `parse_examples`

Add optional method name to `generate_test_code`:
```rust
pub fn generate_test_code(result: &ParseResult, method_name: Option<&str>) -> String
```

When `method_name` is `Some`, generate uncommented assertions:
```rust
let result = Solution::two_sum(nums, target);
assert_eq!(result, expected);
```

When `None`, keep the current `// TODO` comment style.

Existing callers pass `None` to preserve backward compatibility.

### 5. Simplify `./lc`

Keep `cmd_setup` and `cmd_reset`. Remove `cmd_help` and the interactive numbered menu. Update the `case` dispatch:

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

### 6. New Dependency

Add to `xtask/Cargo.toml`:
```toml
ureq = "3"
```

No new deps for the main leetcode crate.

## Testing Strategy

**Unit tests (pure, no I/O):**
- `extract_slug` — valid URLs (with/without trailing slash, with `/description/`, with query params, with fragments), invalid URLs (non-leetcode domains, missing path), edge cases
- `strip_html` — tags, entities, numeric entities, nested tags, `<pre>` blocks, empty input
- `extract_method_name` — standard snippet, edge cases (no `pub fn`, empty)
- `generate_test_code` with `method_name` — verify real assertions generated vs `// TODO` style

**Integration tests (in `solve.rs`):**
- Existing tests updated to pass `None` for URL
- Template assembly logic tested separately from `solve::run` to avoid network dependency: a helper function takes `ProblemData` and returns the template string, testable without I/O

**Manual testing:**
- `cargo solve https://leetcode.com/problems/two-sum/` — verify examples and impl block appear
- `cargo solve` — verify clean template + tip
- `cargo solve https://example.com/not-leetcode` — verify warning + fallback
- Network off + `cargo solve <url>` — verify warning + fallback

## Out of Scope

- Authentication (API works without it for public problems)
- Caching fetched problems
- Supporting non-Rust languages
- Premium/locked problems (they return null content — we warn and fall back)
