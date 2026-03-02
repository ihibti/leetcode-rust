# Rust Learning Resources for C Programmers

A curated, opinionated list. Ordered roughly by when you should reach for each resource.

---

## Official

**The Rust Book** -- https://doc.rust-lang.org/book/

The canonical starting point. For C programmers, this chapter order cuts through the noise faster than reading front-to-back:

1. Chapters 1--4: Getting started, basic syntax, ownership and borrowing. This is where the biggest mental shift from C happens. Do not skip or skim chapter 4.
2. Chapter 6: Enums and pattern matching. Replaces the C idiom of tagged unions plus switch statements, but safer and more expressive.
3. Chapter 8: Common collections (Vec, String, HashMap). The standard data structures you will use constantly.
4. Chapter 5: Structs. Feels familiar from C, but methods and associated functions change how you organize code.
5. Chapter 10: Generics, traits, and lifetimes. The type system features that have no direct C equivalent. Take your time here.
6. Chapter 13: Iterators and closures. Where Rust starts feeling very different from C. Essential for writing idiomatic code.
7. Chapter 15: Smart pointers (Box, Rc, RefCell). Closest thing to manual memory management patterns you are used to from C.

**Rust by Example** -- https://doc.rust-lang.org/rust-by-example/

Learn-by-doing companion to the Rust Book. Good for quickly checking "how do I do X in Rust" when you already know the concept from C.

**The Rust Reference** -- https://doc.rust-lang.org/reference/

Not a tutorial. Use this when you need the precise specification of how something behaves -- memory layout, operator precedence, trait resolution rules. Comparable to reading the C standard, but more approachable.

---

## Interactive

**Rustlings** -- https://github.com/rust-lang/rustlings

Small exercises that run in your terminal. Fix broken code to make tests pass. Pairs well with LeetCode practice: Rustlings builds Rust fluency, LeetCode builds algorithmic thinking.

**Exercism Rust Track** -- https://exercism.org/tracks/rust

Structured exercises with mentorship available. Problems are more substantial than Rustlings and come with community solutions you can compare against. Good for seeing idiomatic patterns.

**Rust Playground** -- https://play.rust-lang.org/

Browser-based compiler. Useful for quickly testing a snippet without setting up a local file, sharing code in discussions, or checking how different Rust editions behave.

---

## Video

**Let's Get Rusty**

Beginner-friendly video series that follows the Rust Book chapter by chapter. Good if you want a second explanation of a concept that did not click from reading alone.

**Jon Gjengset**

Long, detailed deep dives into real Rust code and crate internals. Watch after you have the basics down. His "Crust of Rust" series covers specific topics (lifetimes, iterators, smart pointers) at a level that will solidify your understanding.

**No Boilerplate**

Short, well-produced videos that explain *why* Rust makes certain design choices. Useful for motivation and building intuition about the language philosophy, not for learning syntax.

**fasterthanlime**

Long-form systems programming content. Covers topics like async, networking, and binary formats at depth. Relevant background for a C programmer who wants to understand how Rust handles the same low-level problems differently.

---

## Books

**"Programming Rust" by Jim Blandy (O'Reilly)**

The best book for C and C++ programmers transitioning to Rust. Assumes systems programming experience and explains Rust concepts in terms you already understand. Covers ownership, concurrency, and unsafe Rust thoroughly.

**Rust Design Patterns** -- https://rust-unofficial.github.io/patterns/

Free, online. A collection of idiomatic Rust patterns and anti-patterns. Useful once you can write Rust that compiles but want to learn how to write Rust that experienced developers would recognize as clean.

---

## Community

**r/rust and r/learnrust**

The main subreddit (r/rust) covers news, libraries, and discussion. r/learnrust is specifically for questions, no matter how basic. Both are welcoming to beginners.

**Rust Discord**

Active chat with dedicated channels for beginners. Good for getting quick answers when you are stuck on a compiler error.

**This Week in Rust** -- https://this-week-in-rust.org/

Weekly newsletter covering new crates, blog posts, RFCs, and job postings. Subscribe once you are comfortable with the basics to stay current with the ecosystem.
