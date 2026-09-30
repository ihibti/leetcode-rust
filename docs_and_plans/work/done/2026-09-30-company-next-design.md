# `cargo next` — Company-Wise Problem Picker — Design

- **Date:** 2026-09-30
- **Status:** Implemented
- **Scope:** `xtask` tooling, archive header format, one-time archive reset
- **Branch:** `company-next`

## Context

The workspace drives LeetCode practice through an `xtask` CLI
(`cargo solve <url>` / `archive` / `progress`). Problem selection is manual:
the user finds a URL and passes it to `solve`.

The goal is a fresh, interview-focused grind: draw problems from the
company-tagged lists LeetCode Premium exposes ("problems Amazon interviewees
reported in the last 30 days", etc.), skipping what has already been solved.
The existing archive is discarded so previously solved but forgotten problems
come back into rotation.

### Data source

[liquidslr/leetcode-company-wise-problems](https://github.com/liquidslr/leetcode-company-wise-problems)
publishes one folder per company (470 companies) with five CSVs:

| Window | File                              | Amazon rows (Aug 2026) |
|--------|-----------------------------------|------------------------|
| `30d`  | `1. Thirty Days.csv`              | 171                    |
| `3m`   | `2. Three Months.csv`             | 451                    |
| `6m`   | `3. Six Months.csv`               | 726                    |
| `6m+`  | `4. More Than Six Months.csv`     | 1634                   |
| `all`  | `5. All.csv`                      | 2012                   |

Schema: `Difficulty,Title,Frequency,Acceptance Rate,Link,Topics`.
`Title` and `Topics` are quoted when they contain commas (`"Pow(x, n)"`).
`Difficulty` is upper-case (`EASY`). `Frequency` is LeetCode's relative score
(top problem = 100; Amazon 30d ranges 39.1–100, 89/171 rows at the 39.1 floor).
`Acceptance Rate` is wrong upstream (Two Sum = `0.0058`) and is ignored.

Windows are relative to the **snapshot date**, not today. Snapshots are
irregular (Jun 2025 → Jun 2026 → 16 Aug 2026), so the CSV is fetched fresh
on every run to pick up new snapshots automatically.

[snehasishroy/leetcode-companywise-interview-questions](https://github.com/snehasishroy/leetcode-companywise-interview-questions)
was evaluated and rejected as default: older snapshot (12 Jul 2026), fewer
Amazon rows, 5-bucket frequency, no topics. It has more companies (660) and a
correct acceptance column, so the design keeps sources pluggable.

### Facts from the current code that shape the design

- `fetch_problem` (`xtask/src/leetcode.rs`) reports a premium problem as
  `InvalidResponse("missing content (premium problem?)")`, indistinguishable
  from a malformed response. Verified: GraphQL returns
  `{"isPaidOnly":true,"content":null,"codeSnippets":null}` for `alien-dictionary`.
- `solve::run` falls back to a blank template on any fetch failure, so routing
  `next` through it unchanged would turn premium picks into empty files.
- "Done" can currently only be inferred from archive filenames, which derive
  from the free-text `cargo archive <name>` argument.
- `rand` is a dependency of the solutions crate, not of `xtask`.

## Goals

- `cargo next [window] [-c company] [-d difficulty] [--force]` draws a
  frequency-weighted random problem not yet archived, from the chosen window,
  and prepares `src/solution.rs` exactly as `cargo solve <url>` does.
- Premium and non-Rust problems are skipped transparently.
- `archive/` is the single ledger of solved problems.
- Selection logic is pure and deterministic under test.

## Non-goals

- No local catalogue cache or offline mode.
- No persistence of skipped (premium) slugs between runs.
- No coverage/statistics output in `next`, no `--dry-run`.
- No second source implementation (only the extension point).
- No `cargo reset` command; the reset is a one-time manual step.
- No change to archive privacy/tracking policy (the June design's submodule
  work remains separate).

---

## Components

```
xtask/src/
  catalog.rs    NEW   Problem, Difficulty, Window, Source, CatalogError
                      csv_url(Source, company, Window) -> String
                      parse_csv(Source, &str) -> Result<Catalog, CatalogError>
  next.rs       NEW   pure:  candidates, weight, pick_weighted
                      shell: run(root, &NextArgs, &impl Remote, rolls) -> Result<Outcome, NextError>
  remote.rs     NEW   trait Remote + UreqRemote
  solve.rs      MOD   ensure_free(root, force); start(root, Option<&ProblemData>)
  leetcode.rs   MOD   isPaidOnly in query; FetchError::PaidOnly; parse_problem_response
  archive.rs    MOD   optional name; `//! Slug:` header
  progress.rs   MOD   ProblemMeta.slug; load_archive; done_slugs
  main.rs       MOD   Next subcommand
xtask/Cargo.toml       + csv = "1", rand = "0.8"
.cargo/config.toml     + next = "run -p xtask -- next"
```

### `catalog`

```rust
pub enum Source { Liquidslr }
pub enum Window { Days30, Months3, Months6, Older, All }
pub enum Difficulty { Easy, Medium, Hard }

pub struct Problem {
    pub slug: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub frequency: f64,
    pub topics: Vec<String>,
}

pub struct Catalog {
    pub problems: Vec<Problem>,
    pub skipped_rows: usize,
}

pub enum CatalogError {
    SchemaChanged { expected: String, found: String },
    Csv(String),
}
```

- `Problem` is the boundary: nothing downstream of `parse_csv` knows which
  source produced it. A new source is a new `Source` variant with its own
  `csv_url` and row-mapping arms.
- `Window` CLI values: `30d`, `3m`, `6m`, `6m+`, `all` (clap `ValueEnum`,
  default `30d`).
- `Difficulty` parsing is case-insensitive (CSV `EASY`, CLI `easy`,
  GraphQL `Easy`).
- The slug is extracted from the `Link` column via the existing
  `leetcode::extract_slug`, never derived from `Title`.
- `csv_url` percent-encodes each path segment (company folder and file name),
  leaving only RFC 3986 unreserved characters (`A–Z a–z 0–9 - . _ ~`) as-is:
  `Goldman Sachs` → `Goldman%20Sachs`, `1. Thirty Days.csv` →
  `1.%20Thirty%20Days.csv`. The encoder is a small pure function in
  `catalog`. The company is passed through case-sensitively; liquidslr folder
  names are capitalised (`Amazon`).
- Parsing uses the `csv` crate (quoted fields are the failure mode of
  hand-rolled splitting).
- The header row must match the expected column list exactly, otherwise
  `SchemaChanged`. A data row with an unparseable link, difficulty, or
  frequency is skipped and counted in `skipped_rows`.

### `next` (pure part)

```rust
pub fn candidates<'a>(
    problems: &'a [Problem],
    done: &HashSet<String>,
    difficulty: Option<Difficulty>,
) -> Vec<&'a Problem>;

pub fn weight(problem: &Problem) -> f64;

pub fn pick_weighted<'a>(pool: &[&'a Problem], roll: f64) -> Option<&'a Problem>;
```

- `weight` is linear in `frequency` by default, clamped to a small positive
  minimum so no candidate has zero probability. It is isolated so the
  preference strength can be tuned (e.g. `freq²` ≈ 6.5× top-vs-floor instead
  of ≈ 2.6×) without touching the sampler.
- `pick_weighted` maps `roll ∈ [0, 1)` onto cumulative weights. `roll = 0.0`
  selects the first element, `roll → 1.0` the last; an empty pool yields
  `None`. It takes a number, not an RNG, so tests use exact boundary values.

### `remote`

```rust
pub trait Remote {
    fn get_text(&self, url: &str) -> Result<String, RemoteError>;
    fn problem(&self, slug: &str) -> Result<ProblemData, FetchError>;
}
```

- `RemoteError` distinguishes `NotFound` (HTTP 404) from other failures.
- `UreqRemote` implements both with `ureq` (same agent settings as today) and
  delegates `problem` to `leetcode::fetch_problem`.
- This is the seam the 2026-06-17 design called `LeetCodeClient`, introduced
  with only the surface `next` needs; `submit` can extend it later.

### `leetcode`

- The GraphQL query adds `isPaidOnly`.
- Response handling moves out of `fetch_problem` into
  `parse_problem_response(slug, body) -> Result<ProblemData, FetchError>`
  (pure). Order of checks: `isPaidOnly == true` → `FetchError::PaidOnly`;
  missing title → `InvalidResponse`; missing content → `InvalidResponse`;
  no Rust snippet → `NoRustSnippet`.

### `solve`

- `ensure_free(root, force) -> Result<(), io::Error>` holds the existing
  unsaved-work check.
- `start(root, Option<&ProblemData>)` writes `solution.rs` (assembled
  template or blank) and `.solve_session`.
- `run(root, force, url)` becomes: `ensure_free` → resolve URL to
  `Option<ProblemData>` (unchanged fallback behaviour) → `start`.
  Observable behaviour of `cargo solve` is unchanged.

---

## Runtime flow

```
cargo next 3m -d medium
 │
 ├─1 guard      solve::ensure_free(root, force)
 ├─2 done set   progress::done_slugs(root/archive)
 ├─3 catalogue  remote.get_text(csv_url(..)) → catalog::parse_csv(..)
 ├─4 pool       next::candidates(problems, done, difficulty)
 ├─5 draw loop  pick_weighted(pool, rolls()) → remote.problem(slug)
 │                Ok(data)             → break
 │                Err(PaidOnly)        → print skip line, drop from pool, redraw
 │                Err(NoRustSnippet)   → print skip line, drop from pool, redraw
 │                Err(Network|Invalid) → abort
 ├─6 write      solve::start(root, Some(&data))
 └─7 report     "<Difficulty> · freq <f> · <Title>\n  <link>"
```

The guard runs before any network access. Each skip removes the candidate from
the pool, so the loop terminates after at most `pool.len()` fetches.

### Outcomes and errors

```rust
pub enum Outcome {
    Picked(Problem),
    Exhausted(Exhaustion),
}

pub struct Report {
    pub outcome: Outcome,
    pub skipped: Vec<Skip>,
    pub malformed_rows: usize,
}

pub enum Exhaustion {
    AllDone { total: usize },
    FilterEmpty { difficulty: Difficulty, done: usize },
    AllUnavailable { skipped: usize },
}

pub enum NextError {
    Unsaved(io::Error),
    CompanyNotFound(String),
    Catalog(CatalogError),
    Network(String),
    Fetch(FetchError),
    Io(io::Error),
}
```

| Situation                                 | Result                                    | Exit |
|-------------------------------------------|-------------------------------------------|------|
| Unsaved `solution.rs`, no `--force`       | `Unsaved`                                 | 1    |
| CSV 404                                   | `CompanyNotFound` ("names are case-sensitive") | 1 |
| CSV header differs                        | `Catalog(SchemaChanged)`                  | 1    |
| Some malformed rows                       | one warning line with the count           | —    |
| Pool empty, no filter                     | `Exhausted(AllDone)` → "try `cargo next 3m`" | 0 |
| Pool empty because of `-d`                | `Exhausted(FilterEmpty)` → "drop -d or widen" | 0 |
| Every remaining candidate premium/non-Rust| `Exhausted(AllUnavailable)`               | 0    |
| GraphQL network/invalid response          | `Network` / `Fetch`, workspace untouched  | 1    |

Mapping rules:

- CSV fetch: `RemoteError::NotFound` → `CompanyNotFound`; any other
  `RemoteError` → `Network`.
- Problem fetch: `PaidOnly` / `NoRustSnippet` are consumed by the draw loop;
  `FetchError::Network` and `FetchError::InvalidResponse` → `Fetch`.
- Exhaustion precedence: if every problem in the window is archived →
  `AllDone` (regardless of `-d`); otherwise, if the difficulty filter leaves
  nothing → `FilterEmpty`, where `done` counts archived problems of that
  difficulty in the window; if the pool was drained by skips →
  `AllUnavailable`.

`NextError` is converted to the `io::Error` that `main` reports only at the
`main.rs` boundary, so tests can match variants.

`run` takes `rolls: impl FnMut() -> f64`; `main` passes `rand::random::<f64>`.

---

## Archive and progress

### Header format

```
//! Problem: merge-intervals
//! Slug: merge-intervals
//! Difficulty: Medium
//! Tags: Array,Sorting
//! Date: 2026-09-30
//! Time: 34m
```

- `Slug:` is written only when `.solve_session` carries a slug, and its value
  always comes from the session.
- `cargo archive [name]`: name resolution is explicit argument → session slug →
  error ("no name given and no session slug; pass a name"). The name only
  drives the filename (`normalize_name`) and the `Problem:` line.
- An archive without `Slug:` (blank `cargo solve`) never counts as done for
  any company list.

### Progress

- `load_archive(dir) -> io::Result<Vec<ProblemMeta>>` holds the directory walk
  and `parse_meta` loop currently inside `progress::run`; missing directory →
  empty vector.
- `progress::run` and `done_slugs(dir) -> io::Result<HashSet<String>>` both use
  it, keeping a single header parser.
- `ProblemMeta` gains `slug`; `parse_meta` reads `//! Slug: `.

---

## One-time reset

Because `archive/` is the ledger, starting a fresh grind means emptying
`archive/` (and resetting `src/solution.rs` / `.solve_session`). No command is
provided for it.

---

## Testing

### Pure units

| Module     | Cases |
|------------|-------|
| `catalog`  | `csv_url` for all five windows; company with a space is encoded. `parse_csv` on a captured liquidslr fixture containing a quoted title with a comma and quoted topics. `EASY/MEDIUM/HARD` parse. Header mismatch → `SchemaChanged`. One malformed row → skipped, `skipped_rows == 1`. |
| `next`     | `candidates` excludes done slugs; applies difficulty filter. `pick_weighted`: `0.0` → first; `0.999…` → last; mid roll → correct cumulative bucket; empty → `None`. `weight` is positive for zero/negative frequency. |
| `leetcode` | `parse_problem_response` on the captured `alien-dictionary` payload → `PaidOnly`; on a normal payload → `ProblemData`; no Rust snippet → `NoRustSnippet`. |
| `progress` | `parse_meta` reads `Slug:`. `done_slugs`: missing dir, empty dir, mix of files with and without `Slug:`. |
| `archive`  | Name omitted → session slug used. `Slug:` written from session. No session slug → no `Slug:` line. Explicit name + session slug → filename from name, `Slug:` from session. Name omitted and no session slug → error. |
| `solve`    | Existing tests stay green after the `ensure_free`/`start` split. |

### Through the `Remote` seam

`FakeRemote` returns canned CSV and per-slug results and records every call.
`TempDir` workspaces.

- Happy path: `solution.rs` contains the drawn problem's snippet;
  `.solve_session` carries its slug.
- Reroll: first draw `PaidOnly`, second `Ok` → second slug written.
- GraphQL network error → `NextError::Network`; `solution.rs` byte-identical.
- All candidates `PaidOnly` → `Exhausted(AllUnavailable)`; nothing written.
- Dirty `solution.rs`, no `--force` → `Unsaved`; fake records **zero** calls.
- Archived slug present → never drawn.

### Manual (supervisor)

1. `cargo next` → a problem is written; `cargo test` compiles the generated
   tests.
2. Solve, `cargo archive` with no name, repeat `cargo next` several times →
   the archived slug does not reappear.
3. `cargo next -c google` → readable company-not-found error;
   `cargo next -c Google` works.
4. `cargo next 30d -d hard` a few times → occasional `skip … (premium)` lines.
5. `cargo solve <url>` and `cargo progress` behave as before.

`cargo test` and `cargo clippy` stay green throughout.

## Documentation

- README: add `cargo next` to the command table and the workflow diagram;
  note that `cargo archive` no longer needs a name after `solve`/`next`.
- `docs_and_plans/guides/archive-and-next.md`: archive header format as the
  done-ledger contract, and the `cargo next` pipeline.

## Resolved decisions

- Done state: `archive/` is the ledger, matched on `//! Slug:`.
- Selection: frequency-weighted random; weight function isolated.
- Flags: window, `--company` (default `Amazon`), `--difficulty`, `--force`.
- Source: liquidslr, behind the `Problem` boundary.
- Approach A: select → validate → hand off to `solve::start`.

## Open questions

None blocking.
