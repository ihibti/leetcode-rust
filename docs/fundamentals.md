# Rust Syntax for C Programmers — Quick Reference

Things that look weird the first time you see them. For deeper patterns, see `cheatsheet.md`.

---

## `::` vs `.`

In C, `.` accesses struct members. In Rust, both `::` and `.` exist but serve different roles.

```rust
// :: is the path separator — used for modules, associated functions, enum variants
use std::collections::HashMap;
let m = HashMap::new();          // HashMap::new() is an associated function (like a "static method")
let v = Option::Some(5);         // enum variant

// . calls methods on a value
let len = v.unwrap();
let s = String::from("hello");
let upper = s.to_uppercase();    // method call on the String value
```

**Rule of thumb:** `::` navigates to something (a module, a type, a function). `.` calls something on a value you already have.

---

## `&` and `*` — Not Like C

In C, `&` takes an address and `*` dereferences a pointer. Rust reuses the same symbols but with different semantics.

```c
// C
int x = 5;
int *p = &x;    // p is a raw pointer
*p = 10;        // write through pointer
```

```rust
// Rust
let x = 5;
let r = &x;      // r is a reference (not a raw pointer)
let v = *r;      // dereference to get the value

fn add_one(n: &i32) -> i32 {     // borrows, does not own
    *n + 1
}
```

References in Rust are checked at compile time. They cannot be null, cannot dangle, and cannot alias a mutable reference.

---

## `let` — Immutable by Default

```rust
let x = 5;       // immutable — cannot reassign
let mut y = 5;   // mutable — can reassign
y = 10;

// x = 10;       // compile error
```

In C everything is mutable unless you add `const`. Rust flips the default.

---

## `&[T]` vs `[T; N]` vs `Vec<T>`

```rust
let array: [i32; 3] = [1, 2, 3];     // fixed size, on the stack (like C array)
let vec: Vec<i32> = vec![1, 2, 3];    // growable, on the heap (like malloc'd array)
let slice: &[i32] = &vec[1..3];       // borrowed view into contiguous memory (like pointer + length)
```

Most functions take `&[T]` (slice) because it works with both arrays and Vecs.

---

## `Option<T>` and `Result<T, E>`

There is no `NULL` in Rust. There is no `errno`.

```rust
// Option: value might be absent
let found: Option<i32> = vec.iter().find(|&&x| x == 5).copied();
match found {
    Some(val) => println!("got {val}"),
    None => println!("not found"),
}

// Result: operation might fail
let parsed: Result<i32, _> = "42".parse();
match parsed {
    Ok(n) => println!("parsed {n}"),
    Err(e) => println!("failed: {e}"),
}
```

---

## `impl` Blocks

Methods are defined outside the struct definition, inside `impl` blocks.

```rust
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {    // associated function (no self)
        Point { x, y }
    }

    fn distance(&self) -> f64 {         // method (takes &self)
        ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt()
    }
}

let p = Point::new(3, 4);   // ::new — associated function
let d = p.distance();       // .distance() — method
```

---

## Macros — The `!`

If it ends with `!`, it is a macro, not a function.

```rust
println!("hello");         // macro — can accept format strings
vec![1, 2, 3];             // macro — creates a Vec with initial values
assert_eq!(a, b);          // macro — shows both values on failure
format!("x = {x}");       // macro — returns a String
```

Macros can do things functions cannot: variable argument counts, code generation, compile-time string formatting.

---

## `use`, `mod`, `crate::`

In C you have `#include`. Rust has a module system.

```rust
mod types;                          // declares a module (loads types.rs)
use crate::types::ListNode;        // brings a type into scope
use std::collections::HashMap;     // from the standard library
```

- `crate::` — root of the current project
- `super::` — parent module
- `self::` — current module

---

## `String` vs `&str`

```rust
let owned: String = String::from("hello");   // heap-allocated, owned, growable
let borrowed: &str = "hello";                // string literal, borrowed, immutable
let also_borrowed: &str = &owned;            // borrow a String as &str

fn greet(name: &str) {                       // prefer &str in function params
    println!("hello {name}");
}

greet(&owned);       // works
greet(borrowed);     // works
```

In C, all strings are `char*`. In Rust, `String` owns data, `&str` borrows it.

---

## `std` — The Standard Library

Rust's standard library is at `std::`. Common modules:

```rust
use std::collections::{HashMap, HashSet, VecDeque, BinaryHeap};
use std::cmp::{min, max, Ordering};
use std::rc::Rc;
use std::cell::RefCell;
```

Browse docs at https://doc.rust-lang.org/std/ — press `K` in Neovim on any std type to see its docs.
