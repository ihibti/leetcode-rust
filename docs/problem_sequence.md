# LeetCode Problem Sequence for C Programmers Learning Rust

This sequence is ordered by Rust concept progression, not by LeetCode difficulty. Each problem is chosen because it naturally introduces or reinforces a specific Rust concept, building on what the previous problems established. A C programmer working through this list will encounter ownership, borrowing, smart pointers, and idiomatic Rust patterns in a deliberate order rather than all at once.

| # | Problem | LC# | Difficulty | Rust Concepts Introduced | Why This Order |
|---|---------|-----|------------|--------------------------|----------------|
| 1 | Two Sum | 1 | Easy | `HashMap`, basic syntax, `Vec` | Start here: simple problem, introduces a key data structure and Rust fundamentals |
| 2 | Contains Duplicate | 217 | Easy | `HashSet`, iterators | Builds on HashMap knowledge, introduces set semantics and basic iterator usage |
| 3 | Valid Palindrome | 125 | Easy | `String`, `chars()`, iterator chains | First string manipulation; C programmers must unlearn null-terminated char arrays |
| 4 | Best Time to Buy/Sell Stock | 121 | Easy | Iterators, `fold`, `min`/`max` | Iterator mastery: chaining, folding, and functional-style traversal |
| 5 | Valid Parentheses | 20 | Easy | `Vec` as stack, `match`, `chars()` | Pattern matching introduction; `match` replaces C switch statements entirely |
| 6 | Merge Two Sorted Lists | 21 | Easy | `Option<Box<T>>`, ownership, `mem::swap` | First real ownership challenge; linked lists in Rust are nothing like C |
| 7 | Reverse Linked List | 206 | Easy | Ownership transfer, `Option` manipulation | Deepens ownership understanding by forcing moves through a list |
| 8 | Linked List Cycle | 141 | Easy | References, two-pointer technique | Borrowing vs owning: cycle detection requires reasoning about lifetimes |
| 9 | Binary Tree Inorder Traversal | 94 | Easy | `Rc<RefCell<T>>`, recursion | Smart pointers introduction; shared ownership for tree structures |
| 10 | Max Depth of Binary Tree | 104 | Easy | Pattern matching on `Option`, recursion | Solidifies comfort with `Rc<RefCell<T>>` through a simpler tree problem |
| 11 | Invert Binary Tree | 226 | Easy | `Rc<RefCell<T>>`, tree mutation | Mutating through `RefCell`: interior mutability in practice |
| 12 | Implement Queue using Stacks | 232 | Easy | Struct design, `impl` methods, `Vec` | Custom types and methods; designing data structures the Rust way |
| 13 | Group Anagrams | 49 | Medium | `HashMap<String, Vec<String>>`, sorting, `collect` | Complex nested collections; heavy use of iterator transformations |
| 14 | Top K Frequent Elements | 347 | Medium | `HashMap`, `BinaryHeap`, iterator composition | Combining heap with iterators; stdlib data structures working together |
| 15 | LRU Cache | 146 | Medium | `HashMap` + custom struct, API design | Capstone problem: ties together multiple concepts into a cohesive design |
