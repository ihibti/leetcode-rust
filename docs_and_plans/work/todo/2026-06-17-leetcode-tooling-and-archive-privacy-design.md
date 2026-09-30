# LeetCode Tooling & Archive Privacy — Design

- **Date:** 2026-06-17
- **Status:** Proposed. Piece 3 (archive privacy) is superseded: solutions now
  live on a separate branch pushed only to a private remote.
- **Scope:** `xtask` tooling + repository/git topology

## Context

The workspace is a Rust LeetCode practice harness driven by an `xtask` CLI
(`cargo solve` / `archive` / `progress`). Three changes are wanted:

1. Surface the problem **description** into the working file on `solve`.
2. Add a `cargo submit` command that gets a solution onto LeetCode.
3. Make the repo's **tooling publicly shareable** while keeping the user's
   solved solutions (`archive/`) **private**, without losing the ability to
   `git clone` once and resume work on any machine.

Reading the current code surfaced two facts that shape the design:

- `fetch_problem` (`xtask/src/leetcode.rs`) already fetches the full problem
  HTML and `strip_html`s it into `examples_text`; `assemble_template`
  (`xtask/src/solve.rs`) then **discards** it, using the text only to generate
  tests. So "fetch the description" is really "stop discarding it."
- The GraphQL query does **not** request `questionId`. LeetCode's submit
  endpoint requires it, so `submit` is impossible without extending the query.

## Goals

- Embed the fetched description into `src/solution.rs` (and thus into archived
  files) as a doc-comment header.
- `cargo submit` with a robust, credential-free core and an opt-in full-auto
  layer (programmatic submit + verdict polling).
- Public tooling repo; `archive/` is a git submodule backed by a separate
  **private** repo. One `git clone --recurse-submodules` restores everything on
  an authenticated machine; strangers get the tooling and an empty/denied
  submodule.
- Zero-friction daily workflow: the submodule's existence should be invisible
  during normal use.

## Non-goals

- No browser-cookie auto-extraction (rejected: OS/browser-specific, fragile).
- No auto-inlining of LeetCode-provided `ListNode`/`TreeNode` definitions in v1.
- No public mirror automation (the public repo *is* the tooling repo).

## Hard constraints

- **Portability.** The tooling must work regardless of what the cloned repo
  folder is named, and regardless of absolute location. All paths are derived
  relative to the workspace root via `workspace_root()`
  (`env!("CARGO_MANIFEST_DIR")` → `.parent()`), which is already name-agnostic.
  Any new path (e.g. the auth file) must follow the same rule. The submodule
  path is relative (`archive`) in `.gitmodules`; only the submodule *URL* is
  absolute, and that is per-user and documented.
- **Secrets never enter git.** Auth lives in a gitignored file or env vars and
  is re-established per machine.
- **Testability.** New logic is split into pure functions (parsing, extraction,
  formatting) plus a thin side-effectful shell. Network access sits behind a
  trait seam so orchestration is testable with a fake.

---

## Piece 1 — Surface the description

**Change:** `assemble_template` prepends `data.examples_text` as a `//!`
doc-header above the snippet.

Resulting `src/solution.rs` shape:

```
//! <problem title> (<difficulty>)
//!
//! <stripped problem statement, examples, constraints>

use crate::types::*;

pub struct Solution;

impl Solution { ... }        // fetched snippet

#[cfg(test)]
mod tests { ... }            // generated from examples
```

- The header is `//!` (inner doc) so it stays attached to the file and travels
  into the archived copy on `cargo archive`, making each archived file a
  self-contained record (problem + solution + timing metadata).
- Default: embed the full `examples_text`. If this proves noisy it can be
  trimmed to the statement only — a later, low-risk tweak.

**Testability:** the header assembly is a pure `String -> String` transform;
extend the existing `assemble_template` tests to assert the header is present
and that `extract_submission` (Piece 2) can strip it back off.

---

## Piece 2 — Tiered `cargo submit`

New subcommand `Submit` in `xtask/src/main.rs`; new module `xtask/src/submit.rs`.
Operates on the **current** `src/solution.rs`; the problem slug comes from
`.solve_session`.

### Robust core (no credentials — works for any clone)

- **`extract_submission(src: &str) -> String`** (pure): strips
  `use crate::types::*;`, `pub struct Solution;`, the `//!` header, and the
  `#[cfg(test)] mod tests { ... }` block, leaving the bare `impl Solution { ... }`
  (plus any free-standing helper items the user wrote). Fully unit-tested.
- Copies the extracted code to the clipboard and opens the problem's submit
  page in the browser (cross-platform crates — clipboard + opener — chosen at
  plan time; both are small and portable).
- **`ListNode`/`TreeNode`:** not special-cased. LeetCode provides its own
  identically-shaped types, so the bare `impl Solution` compiles there once the
  local `use crate::types::*;` is stripped. Any genuine mismatch surfaces as
  LeetCode's own "Compile Error" verdict via the poller — authoritative and
  self-explanatory, no speculative warning needed.

### Full-auto layer (only when a token is configured)

- **`questionId`:** extend the GraphQL query in `fetch_problem` to request it;
  add `extract_question_id` (mirrors the existing `extract_json_string`
  helpers). `ProblemData` gains a `question_id` field, persisted into
  `.solve_session` so `submit` need not re-fetch.
- **`LeetCodeClient` trait (seam):** wraps the HTTP calls
  (`fetch`, `submit`, `check`). The real impl uses `ureq` (as `fetch_problem`
  does today); tests use a fake. This also lets the existing fetch path be
  exercised without network.
- **Flow:** POST `typed_code` + `lang=rust` + `question_id` to
  `/problems/{slug}/submit/` with the session cookie + `csrftoken`; poll
  `/submissions/detail/{id}/check/` until `state == "SUCCESS"`; parse the
  verdict.
- **`parse_verdict(json) -> Verdict`** (pure): Accepted / Wrong Answer /
  TLE / Compile Error / Runtime Error, plus runtime, memory, and the failing
  input where present. Unit-tested against captured sample payloads.

### Credentials

- Resolution order: env (`LEETCODE_SESSION`, `LEETCODE_CSRF`) → gitignored
  `leetcode-auth.json` at the workspace root (located via `workspace_root()`).
- **`cargo submit --login`**: prompts for the two values (with a one-line
  pointer to where they live in browser devtools) and writes the gitignored
  auth file. Removes any doc-hunting from setup.
- If neither env nor file is present, `submit` runs the robust core and prints a
  single friendly line explaining how to enable full-auto. It never hard-fails
  for lack of credentials.

---

## Piece 3 — `archive/` as a private submodule

### Topology

- Public repo: the tooling (this repo, under any folder name).
- Private repo: e.g. `leetcode-archive`, holding the solved `*.rs` files.
- `archive/` is a git submodule in the public repo pointing at the private repo.
  `.gitmodules` stores the relative path `archive` and the private URL.

### One-time migration

1. Create the private repo.
2. Move the existing `archive/*.rs` into it; commit; push.
3. In this repo: `git submodule add <private-url> archive`; commit `.gitmodules`
   + the gitlink.

(The current `archive/` files are untracked, so no `git rm --cached` of
committed history is required — they were never committed here.)

### Clone story

- Authenticated machine: `git clone --recurse-submodules <public-url>` restores
  tooling **and** answers. `git submodule update --init` after a plain clone is
  the fallback.
- Stranger / forker: plain clone yields the tooling; the submodule fetch is
  denied or skipped, leaving `archive/` empty. To use their own archive they
  repoint the submodule URL — documented in the README.

### Zero-friction archiving

`cargo archive <name>` is extended so the submodule is invisible in daily use:

1. Write the archived file into `archive/` (as today).
2. In the submodule: stage, commit (message derived from the problem), and
   `git push`.
3. In the parent: stage the updated gitlink and commit the pointer bump.
4. `--no-push` skips the network steps (offline); the local commits still keep
   the submodule pointer consistent, avoiding the confusing "dirty submodule"
   state.

The git operations are shelled out via `std::process::Command` behind a small
wrapper so the path-resolution and message-formatting logic remains unit-
testable; the network/push steps are guarded by `--no-push` and exercised
manually.

### Graceful empty archive

Anything reading `archive/` (`cargo progress`) must tolerate a missing/empty
directory (stranger clone, or before `submodule update`). Covered by a test
with an empty `TempDir`.

---

## Cross-cutting

### `.gitignore` additions

- `leetcode-auth.json` — credentials.

`index.html` (the tooling's landing page), `docs/`, and the archived files'
*structure* remain public.

### README

Document: clone with `--recurse-submodules`; how to point the submodule at your
own private archive; how `cargo submit` works (core vs `--login` full-auto); and
that everything is relative to the workspace root, so the folder may be named
anything.

---

## Testing strategy

| Unit (pure)                              | Seam / fake          | Manual            |
|------------------------------------------|----------------------|-------------------|
| description header assembly              | submit via fake      | real submit       |
| `extract_submission` (incl. list/tree)   | `LeetCodeClient`     | `--login` write   |
| `extract_question_id`                    | poll/verdict via fake| submodule push    |
| `parse_verdict` (each status)            |                      | `--recurse` clone |
| archive message/path formatting          |                      |                   |
| `progress` on empty archive              |                      |                   |

Existing tests use `TempDir` and are unaffected by the submodule change.
`cargo test` must stay green throughout.

## Suggested implementation order

1. Piece 1 (description header) — smallest, unlocks the `extract_submission`
   contract for Piece 2.
2. Piece 2 robust core (`extract_submission` + clipboard/open).
3. Piece 2 full-auto (`questionId`, `LeetCodeClient` seam, submit/poll/verdict,
   auth + `--login`).
4. Piece 3 (submodule migration, `cargo archive` automation, README,
   `.gitignore`).

## Resolved decisions

- Submit is **tiered**; auth is **env → gitignored file**, set via `--login`.
- Archive privacy via **private submodule**; `cargo archive` **auto-syncs** it.
- **No** type special-casing; LeetCode's verdict is the authority.
- `index.html` stays public.
- Portability (any folder name, relative paths) is a **hard constraint**.

## Open questions

None blocking. The description-length trim (full vs statement-only) is deferred
as a trivial post-hoc adjustment.
