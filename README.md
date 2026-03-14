# LeetCode in Rust

A local workspace for solving LeetCode problems in Rust with full IDE support (rust-analyzer), solution archiving, and progress tracking. Built for 42 school students learning Rust from C.

---

## Quick Start

```bash
git clone https://github.com/ihibti/leetcode-rust.git
cd leetcode-rust
./lc setup                                              # checks your environment
cargo solve https://leetcode.com/problems/two-sum/      # fetches problem, generates tests
# open src/solution.rs — impl block and tests are ready
cargo watch -x test     # live test feedback as you code
cargo archive two-sum -d easy -t "array,hash-map" -r "iterators,entry-api"
cargo progress          # see your stats
```

## Commands

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

## Workflow

```
./lc setup → cargo solve <url> → edit solution.rs → cargo watch -x test → cargo archive
                  ↑                                                            |
                  └────────────────────────────────────────────────────────────┘
```

When you run `cargo solve <url>`, it fetches the problem from LeetCode, generates the `impl Solution` skeleton and test cases from the examples. Open `src/solution.rs` — the method signature and tests are ready, just fill in the implementation.

Running `cargo solve` without a URL gives a blank template.

## Project Structure

```
leetcode-rust/
├── lc                   # Bootstrap script (setup, reset)
├── src/
│   ├── lib.rs           # Crate root
│   ├── solution.rs      # YOUR WORKING FILE
│   ├── types.rs         # ListNode, TreeNode
│   └── macros.rs        # list![], tree![] test helpers
├── xtask/               # CLI tooling (solve, archive, progress)
├── archive/             # Your solved problems
└── docs/
    ├── fundamentals.md  # Rust syntax quick reference (for C programmers)
    ├── cheatsheet.md    # C-to-Rust patterns in depth
    ├── resources.md     # Learning resources + neetcode roadmap
    └── ai-tutor.md      # Setting up AI assistance
```

## Documentation

- **[Fundamentals](docs/fundamentals.md)** — "What does `::` mean?" Quick answers to Rust syntax that surprises C programmers
- **[Cheatsheet](docs/cheatsheet.md)** — Side-by-side C and Rust patterns for LeetCode
- **[Resources](docs/resources.md)** — Curated learning resources + neetcode problem roadmap
- **[AI Tutor](docs/ai-tutor.md)** — Set up Claude Code or other AI tools as your Rust tutor

## Requirements

- macOS or Linux
- Rust toolchain (installed via [rustup](https://rustup.rs/))
- Recommended: `cargo-watch` for live test feedback

## Credits

Shoutout to [Idrissa](https://github.com/iibabyy) for getting me into Rust.
