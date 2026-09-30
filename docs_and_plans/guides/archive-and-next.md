# Archive format and `cargo next`

## Archive header

Every file in `archive/` starts with `//!` lines that `cargo archive` writes and
`xtask/src/progress.rs::parse_meta` reads. Parsing stops at the first line that
is not `//!`.

```
//! Problem: trapping-rain-water     name given to `cargo archive`, else the session slug
//! Slug: trapping-rain-water        LeetCode slug from .solve_session (absent for blank sessions)
//! Difficulty: Hard
//! Tags: Array,Two Pointers,...
//! Rust concepts: ...               only with -r
//! Date: 2026-09-30
//! Time: 34m
```

`Slug:` is the done-ledger contract: `progress::done_slugs(archive/)` collects
every non-empty `Slug:` value, and `cargo next` never draws a slug in that set.
Only top-level `archive/*.rs` files are read; subdirectories are ignored.

Consequences:

- Renaming or hand-editing an archived file does not affect matching as long as
  the `Slug:` line is intact.
- A solution started with blank `cargo solve` has no slug and never counts as
  done for any company list.
- A fresh grind is emptying `archive/`; there is no other state.

## `cargo next` pipeline

`xtask/src/next.rs::run`, all network access behind `remote::Remote`:

1. `solve::ensure_free` — refuses to overwrite unsaved `solution.rs` (before any
   network call).
2. `progress::done_slugs` — the done set.
3. `catalog::csv_url` + `Remote::get_text` + `catalog::parse_csv` — the company
   list as `Vec<Problem>`. A 404 means the company folder does not exist
   (names are case-sensitive).
4. `next::candidates` — drop done slugs, apply `-d`.
5. `next::pick_weighted` with `rand::random` — weight is
   `next::weight` = `max(frequency, 1.0)`. Tune preference strength there.
6. `Remote::problem` — `PaidOnly` / `NoRustSnippet` drop the candidate and
   redraw; any other error aborts with the workspace untouched.
7. `solve::start` — same workspace writing as `cargo solve <url>`.

## Data source

[liquidslr/leetcode-company-wise-problems](https://github.com/liquidslr/leetcode-company-wise-problems),
fetched fresh each run from `main`. Schema
`Difficulty,Title,Frequency,Acceptance Rate,Link,Topics`; a header change
fails loudly (`SchemaChanged`). The `Acceptance Rate` column is wrong upstream
and unused. Windows are relative to the upstream snapshot date.

Adding a source: new `catalog::Source` variant with its own arms in `csv_url`
and `parse_csv`, mapping rows to `Problem`. Nothing downstream changes.
