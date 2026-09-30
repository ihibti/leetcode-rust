# `cargo next` Implementation Plan

> Execution: native, in this session, task by task. Each task is test-first and
> ends with `cargo test -p xtask` green and a commit on `company-next`.

**Goal:** `cargo next [window] [-c company] [-d difficulty] [--force]` draws a
frequency-weighted random, not-yet-archived problem from a company list and
prepares `src/solution.rs` like `cargo solve <url>`.

**Architecture:** Pure selection core (`catalog` parsing, `next` candidate
filtering and weighted pick) behind a `Remote` seam; `solve` split into
`ensure_free` + `start` so `next` reuses workspace writing; `archive/` is the
done-ledger via a `//! Slug:` header.

**Tech stack:** Rust 2024, clap 4.6, ureq 3.2, chrono; new: `csv = "1"`,
`rand = "0.8"` (xtask only).

**Spec:** `docs_and_plans/work/todo/2026-09-30-company-next-design.md`

## Global constraints

- Paths derive from `workspace_root()`; no absolute paths in code.
- Observable behaviour of `cargo solve`, `cargo archive <name>`, `cargo progress`
  is unchanged except where the spec says otherwise.
- No network in tests; everything network-bound goes through `Remote`.
- Baseline: 131 xtask tests pass at `3072a15`.
- No inline comments (house style).

## Review focus

1. **Float drift at the top of the roll range** — `roll = 0.999_999_9` must
   return the last candidate, never `None`. Pinned in Task 7.
2. **CRLF line endings / trailing newline in the CSV** — parse identically.
   Pinned in Task 5.
3. **Duplicate `Link` rows in one CSV** — keep the first, so a problem is not
   double-weighted. Pinned in Task 5.
4. **`//! Slug:` with trailing whitespace** — trimmed before matching.
   Pinned in Task 2.
5. **Lower-case company (`-c amazon`)** — `CompanyNotFound` naming the value
   and the case-sensitivity hint, not a generic network error. Pinned in Task 7.

---

### Task 1: `leetcode` — premium detection

**Files:** `xtask/src/leetcode.rs`

**Produces:**
- `FetchError::PaidOnly` (Display: `"Premium-only problem"`)
- `pub fn parse_problem_response(slug: &str, body: &str) -> Result<ProblemData, FetchError>`
- `fetch_problem` = HTTP + `parse_problem_response`; query gains `isPaidOnly`.

Detection: `body.contains("\"isPaidOnly\":true")` → `PaidOnly`, checked first.

**Tests (write first):**
- `response_paid_only` — captured payload
  `{"data":{"question":{"title":"Alien Dictionary","isPaidOnly":true,"content":null,"codeSnippets":null}}}`
  → `Err(PaidOnly)`.
- `response_ok` — payload with title, difficulty, content, topicTags, rust
  snippet, `"isPaidOnly":false` → `Ok` with fields populated.
- `response_no_rust` — snippets without `rust` → `Err(NoRustSnippet)`.
- `response_missing_title` → `Err(InvalidResponse(_))`.

Commit: `Detect premium problems in LeetCode responses`

### Task 2: `progress` — slug and done set

**Files:** `xtask/src/progress.rs`

**Produces:**
- `ProblemMeta.slug: String` parsed from `//! Slug: ` (trimmed).
- `pub fn load_archive(dir: &Path) -> io::Result<Vec<ProblemMeta>>` — missing
  dir → `Ok(vec![])`; only `*.rs` files.
- `pub fn done_slugs(dir: &Path) -> io::Result<HashSet<String>>` — non-empty
  slugs from `load_archive`.
- `run` uses `load_archive`; output unchanged.

**Tests:**
- `parse_slug` — header with `//! Slug: two-sum` → `meta.slug == "two-sum"`.
- `parse_slug_trims` — `//! Slug: two-sum   ` → `"two-sum"`.
- `done_slugs_missing_dir` → empty.
- `done_slugs_empty_dir` → empty.
- `done_slugs_mixed` — one file with slug, one without, one `.txt` → `{slug}`.

Commit: `Expose archived slugs as the done set`

### Task 3: `archive` — optional name, `Slug:` header

**Files:** `xtask/src/archive.rs`, `xtask/src/main.rs`

**Produces:**
- `SessionMeta.slug: Option<String>` via `extract_field(&content, "slug")`.
- `pub fn run(root, name: Option<&str>, difficulty, tags, rust_concepts)`.
  Name = explicit → session slug → `InvalidInput("no name given and no session
  slug; pass a name")`.
- Header: `//! Slug: <session slug>` right after `//! Problem:` when the
  session has a slug.
- CLI: `name: Option<String>`.

**Tests:** existing tests updated to `Some("...")`; new:
- `archive_name_from_session` — session slug `two-sum`, `name=None` →
  `archive/two_sum.rs` with `//! Problem: two-sum` and `//! Slug: two-sum`.
- `archive_explicit_name_keeps_session_slug` — `name=Some("mine")`, session
  slug `two-sum` → `archive/mine.rs` containing `//! Slug: two-sum`.
- `archive_no_session_slug_no_header` — `name=Some("x")`, no session → no
  `//! Slug:` line.
- `archive_no_name_no_session_errors` → `Err`, kind `InvalidInput`.

Commit: `Record session slug in archive header; make name optional`

### Task 4: `solve` — split guard and workspace writing

**Files:** `xtask/src/solve.rs`

**Produces:**
- `pub fn ensure_free(root: &Path, force: bool) -> Result<(), io::Error>`
  (existing check, same message).
- `pub fn start(root: &Path, data: Option<&ProblemData>) -> Result<(), io::Error>`
  — writes `solution.rs` (assembled or template) and `.solve_session`.
- `run` = `ensure_free` → resolve URL (unchanged messages/fallbacks) → `start`.

**Tests:** all existing solve tests pass unchanged; new:
- `start_with_data_writes_snippet_and_session` — `solution.rs` contains the
  snippet, session contains `"slug":"two-sum"`.
- `ensure_free_rejects_dirty` / `ensure_free_accepts_template` /
  `ensure_free_force`.

Commit: `Split solve into ensure_free and start`

### Task 5: `catalog` — source adapter and CSV parsing

**Files:** create `xtask/src/catalog.rs`; `xtask/Cargo.toml` (+`csv = "1"`);
`main.rs` (`mod catalog;`)

**Produces:**
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Source { Liquidslr }
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Window { #[value(name="30d")] Days30, #[value(name="3m")] Months3,
                  #[value(name="6m")] Months6, #[value(name="6m+")] Older, All }
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Difficulty { Easy, Medium, Hard }
pub struct Problem { slug, title, difficulty: Difficulty, frequency: f64, topics: Vec<String> }
pub struct Catalog { problems: Vec<Problem>, skipped_rows: usize }
pub enum CatalogError { SchemaChanged { expected: String, found: String }, Csv(String) }
impl Difficulty { pub fn parse(s: &str) -> Option<Self> }  // case-insensitive
impl Display for Difficulty / Window / CatalogError
pub fn encode_segment(s: &str) -> String
pub fn csv_url(source: Source, company: &str, window: Window) -> String
pub fn parse_csv(source: Source, text: &str) -> Result<Catalog, CatalogError>
```
Base URL:
`https://raw.githubusercontent.com/liquidslr/leetcode-company-wise-problems/main/`.
Expected header: `Difficulty,Title,Frequency,Acceptance Rate,Link,Topics`.
Slug via `leetcode::extract_slug(link)`. Duplicate slugs: keep first.
Topics: split on `,`, trim, drop empty.

**Tests:**
- `encode_space_and_dot` — `"1. Thirty Days.csv"` → `"1.%20Thirty%20Days.csv"`;
  `"AT&T"` → `"AT%26T"`.
- `url_each_window` — five windows map to `1. Thirty Days` … `5. All`.
- `parse_fixture` — 4 real rows incl. `Pow(x, n)` and quoted topics → 4
  problems, correct slug/difficulty/frequency/topics.
- `parse_crlf` — same fixture with `\r\n` → identical result.
- `parse_schema_changed` — snehasishroy header → `SchemaChanged`.
- `parse_bad_row_skipped` — non-numeric frequency → `skipped_rows == 1`.
- `parse_duplicate_link_keeps_first`.
- `difficulty_parse_cases` — `EASY`, `easy`, `Easy` → `Easy`; `x` → `None`.

Commit: `Add company catalogue parsing for liquidslr lists`

### Task 6: `remote` — network seam

**Files:** create `xtask/src/remote.rs`; `main.rs` (`mod remote;`)

**Produces:**
```rust
pub enum RemoteError { NotFound, Other(String) }
pub trait Remote {
    fn get_text(&self, url: &str) -> Result<String, RemoteError>;
    fn problem(&self, slug: &str) -> Result<ProblemData, FetchError>;
}
pub struct UreqRemote;
```
`get_text`: ureq agent (10 s timeout, same user agent); `Error::StatusCode(404)`
→ `NotFound`. `problem` delegates to `leetcode::fetch_problem`.

**Tests:** none beyond compilation (pure network shell; exercised manually).

Commit: folded into Task 7.

### Task 7: `next` — selection and orchestration

**Files:** create `xtask/src/next.rs`; `xtask/Cargo.toml` (+`rand = "0.8"`)

**Produces:**
```rust
pub struct NextArgs { window: Window, company: String, difficulty: Option<Difficulty>, force: bool }
pub fn candidates<'a>(problems: &'a [Problem], done: &HashSet<String>, difficulty: Option<Difficulty>) -> Vec<&'a Problem>
pub fn weight(p: &Problem) -> f64                 // max(frequency, 1.0)
pub fn pick_weighted<'a>(pool: &[&'a Problem], roll: f64) -> Option<&'a Problem>
pub enum Exhaustion { AllDone { total }, FilterEmpty { difficulty, done }, AllUnavailable { skipped } }
pub enum Outcome { Picked(Picked), Exhausted(Exhaustion) }
pub struct Picked { title, slug, difficulty: Difficulty, frequency: f64, skipped: Vec<(String, &'static str)> }
pub enum NextError { Unsaved(io::Error), CompanyNotFound(String), Catalog(CatalogError), Network(String), Fetch(FetchError), Io(io::Error) }
pub fn run(root: &Path, args: &NextArgs, remote: &impl Remote, rolls: impl FnMut() -> f64) -> Result<Outcome, NextError>
```
`run` follows the spec's runtime flow; skip lines are collected in `Picked` /
counted in `AllUnavailable` so `main` prints them (keeps `run` output-free and
testable).

**Tests (pure):**
- `candidates_excludes_done`, `candidates_filters_difficulty`.
- `weight_floor` — frequency 0 → 1.0.
- `pick_first_at_zero`, `pick_last_near_one` (0.999_999_9),
  `pick_mid_bucket` (weights 1,1,2; roll 0.5 → third), `pick_empty_none`.

**Tests (FakeRemote + TempDir):** fake holds `csv: Result<String, RemoteError>`,
`problems: HashMap<slug, Result<ProblemData, FetchError>>`,
`calls: RefCell<Vec<String>>`.
- `run_happy_path` — writes snippet and session slug.
- `run_rerolls_paid_only` — first draw PaidOnly, then OK; `skipped.len()==1`.
- `run_network_error_leaves_workspace` — `solution.rs` byte-identical.
- `run_all_paid_only_exhausted` → `AllUnavailable`.
- `run_dirty_workspace_no_calls` → `Unsaved`, `calls` empty.
- `run_skips_archived` — archive has slug A, catalogue {A,B} → B for any roll.
- `run_all_done` → `AllDone`; `run_filter_empty` → `FilterEmpty`.
- `run_company_not_found` — `NotFound` → `CompanyNotFound("amazon")`.

Commit: `Add cargo next selection and orchestration`

### Task 8: CLI wiring and docs

**Files:** `xtask/src/main.rs`, `.cargo/config.toml`, `README.md`,
create `docs_and_plans/guides/archive-format.md`

- `Command::Next { window: Window (default 30d), -c/--company (default "Amazon"), -d/--difficulty, --force }`.
- `main` calls `next::run(&root, &args, &UreqRemote, rand::random::<f64>)`,
  prints skip lines, the pick report, or the exhaustion message with hints;
  `NextError` → `io::Error::other(e.to_string())`.
- Alias `next = "run -p xtask -- next"`.
- README: command table row, workflow line, optional archive name.
- Guide: header lines, `Slug:` as done-ledger contract, reset = empty
  `archive/`.
- Manual smoke: `cargo next --help`, `cargo next`, `cargo next -c amazon`,
  `cargo next 30d -d hard`; then `cargo solve --force` to restore template.

Commit: `Wire cargo next into the CLI and document it`
