# LeetCode Rust

A local workspace for grinding LeetCode in Rust with full rust-analyzer support. No more fighting the web editor — get completions, docs on hover, type inference, and compiler errors in your own editor.

Built for C programmers learning Rust. Includes a C-to-Rust cheatsheet, curated resources, and a progressive problem sequence ordered by Rust concept.

## Quick Start

```bash
# 1. Clone
git clone <this-repo>
cd <repo-name>

# 2. Check your environment
cargo setup

# 3. Start a new problem
cargo solve

# 4. Open src/solution.rs, paste your impl Solution { ... }

# 5. Run your tests
cargo test
```

## Workflow

```
cargo solve          Edit src/solution.rs          cargo test
    |                        |                         |
    v                        v                         v
 Template reset       Paste impl Solution        Run your tests
                      Write #[test] cases
                             |
                             v
                      cargo archive two-sum -d easy -t "array" -r "HashMap"
                             |
                             v
                      Solution saved to archive/two_sum.rs
                      Template reset, ready for next problem
                             |
                             v
                      cargo progress
                             |
                             v
                      See your stats, difficulty breakdown, concept coverage
```

## Commands

| Command | Description |
|---------|-------------|
| `cargo setup` | Check environment, recommend missing tools |
| `cargo solve` | Reset `src/solution.rs` to a clean template |
| `cargo solve --force` | Reset even if current solution has unsaved work |
| `cargo archive <name>` | Save current solution to `archive/<name>.rs` |
| `cargo archive <name> -d easy -t "array,hash-map" -r "iterators"` | Archive with metadata |
| `cargo progress` | Show solving stats and concept coverage |
| `cargo test` | Run tests for current solution |
| `cargo clippy` | Get idiomatic Rust suggestions |
| `cargo fmt` | Format your code |

## Project Structure

```
leetcode-rust/
├── src/
│   ├── lib.rs          # Crate root
│   ├── types.rs        # ListNode, TreeNode, helper functions
│   ├── macros.rs       # list![], tree![] test helpers
│   └── solution.rs     # YOUR WORKING FILE
├── xtask/              # CLI tooling (solve, archive, progress, setup)
├── archive/            # Your solved problems
│   └── examples/       # Reference solutions (look here for workflow examples)
└── docs/
    ├── cheatsheet.md   # C-to-Rust patterns for LeetCode
    ├── resources.md    # Curated learning resources
    └── problem_sequence.md  # Problems ordered by Rust concept
```

## Test Helpers

The `list!` and `tree!` macros make writing test cases painless:

```rust
// These are already imported in the solution.rs template:
// use crate::types::*;           — ListNode, TreeNode, helpers
// use crate::{list, tree};       — list![] and tree![] macros

#[test]
fn test_merge() {
    let result = Solution::merge_two_lists(list![1, 2, 4], list![1, 3, 4]);
    assert_eq!(list_to_vec(result), vec![1, 1, 2, 3, 4, 4]);
}

#[test]
fn test_tree() {
    let root = tree![3, 9, 20, null, null, 15, 7];
    assert_eq!(Solution::max_depth(root), 3);
}
```

## For C Programmers

If you're coming from C, start with:
1. [`docs/cheatsheet.md`](docs/cheatsheet.md) — side-by-side C vs Rust patterns
2. [`docs/problem_sequence.md`](docs/problem_sequence.md) — problems ordered to progressively introduce Rust concepts
3. [`docs/resources.md`](docs/resources.md) — curated books, videos, and tools

## Optional: Auto-test on Save

Install `cargo-watch` for automatic test reruns whenever you save:

```bash
cargo install cargo-watch
cargo watch -x test
```
