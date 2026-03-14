# Using AI as a Rust Tutor

AI coding assistants work well as tutors when given the right context about who you are and what you are learning. This guide covers how to set one up with this repo.

---

## Claude Code

Claude Code is Anthropic's CLI tool for working with code.

### Setup

1. Install: `npm install -g @anthropic-ai/claude-code`
2. Navigate to this repo: `cd path/to/leetcode-rust`
3. Run: `claude`

Claude Code reads the `CLAUDE.md` file in this repo automatically. It already knows this is a learning environment for 42 students transitioning from C to Rust.

### Learning Output Style

Claude Code has a `learning` output style that changes how it interacts with you:

- Provides educational insights about implementation choices
- Asks you to write key pieces of code yourself (design decisions, algorithms)
- Explains Rust concepts in relation to C when relevant

To enable it, add to your Claude Code settings or run with the appropriate flag. Check `claude --help` for current options.

### Example Prompts

When working on a problem:

- "explain this compiler error" — paste the error, get a breakdown
- "walk me through the ownership in this solution" — understand why borrows/moves happen
- "what Rust concept should I practice next based on my archive?" — personalized progression
- "how would I solve this differently in C vs Rust?" — contrastive learning
- "why does the borrow checker reject this?" — understand the rules, not just the fix

### Example Workflow

```
$ cargo solve
# paste your LeetCode examples when prompted
# open src/solution.rs
$ claude
> I'm working on two-sum. Here's my approach: iterate through the array
> and for each element, check if target - element exists in a HashMap.
> Can you help me write this in Rust?
```

---

## Other AI Tools

If you use Cursor, GitHub Copilot, Gemini CLI, or another AI assistant, you can give it the same context by creating a configuration file.

### AGENTS.md

Create a file called `AGENTS.md` in the repo root with this content:

```markdown
# Context

This is a learning environment for 42 school students practicing LeetCode in Rust.

## User Profile
- Strong C programming background (42 school curriculum)
- New to Rust — learning ownership, borrowing, lifetimes, and idiomatic patterns
- Using this repo to solve LeetCode problems and build Rust fluency
- No sudo access on school machines

## How to Help
- Explain Rust concepts in relation to C equivalents
- Highlight common pitfalls for C programmers (mutability defaults, ownership, no NULL)
- Encourage hands-on practice rather than giving complete solutions
- Help debug compiler errors with explanations, not just fixes
- Reference docs/cheatsheet.md and docs/fundamentals.md for patterns

## Project Structure
- src/solution.rs — the working file for the current problem
- src/types.rs — ListNode, TreeNode definitions
- src/macros.rs — list![], tree![] test helpers
- archive/ — solved problems
- docs/ — learning resources
```

### Cursor

Cursor reads `.cursorrules` in the project root. Create one with similar content to the AGENTS.md above.

### GitHub Copilot

Copilot reads `.github/copilot-instructions.md`. Create one with similar content.

---

## Tips for Learning with AI

1. **Ask "why" before "how"** — understanding the reason behind a pattern teaches more than the pattern itself
2. **Try first, then ask** — attempt the problem for 10-15 minutes before asking for help
3. **Request explanations of compiler errors** — Rust's error messages are famously good; AI can break them down further
4. **Ask for alternative approaches** — there is usually more than one way to solve a problem in Rust
5. **Review suggested code critically** — AI is not always right; understanding why code works is more valuable than having working code
