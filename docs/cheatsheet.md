# C-to-Rust Cheatsheet for LeetCode

---

## 1. Variables & Mutability

**C**
```c
int x = 5;
x = 10;

const int y = 5;
```

**Rust**
```rust
let x = 5;       // immutable by default
let mut x = 5;   // must opt in to mutability
x = 10;

let y: i32 = 5;  // type annotation when needed
```

**Key difference:** Rust variables are immutable by default; add `mut` when you need to reassign or modify in place.

---

## 2. Arrays & Vectors

**C**
```c
int arr[5] = {1, 2, 3, 4, 5};
int len = sizeof(arr) / sizeof(arr[0]);
arr[0] = 10;

int *dyn = malloc(n * sizeof(int));
dyn[0] = 1;
free(dyn);
```

**Rust**
```rust
let arr = [1, 2, 3, 4, 5];           // fixed-size array
let len = arr.len();
let slice = &arr[1..3];              // borrowing a slice: [2, 3]

let mut v = vec![1, 2, 3, 4, 5];    // heap-allocated, growable
v.push(6);
v.pop();                              // returns Option<i32>
v[0] = 10;

let zeroes = vec![0; n];             // n elements, all zero
let first_three = &v[..3];           // slice of a Vec

let matrix = vec![vec![0; cols]; rows];
```

**Key difference:** `Vec<T>` replaces both malloc'd arrays and realloc patterns; slices (`&[T]`) replace pointer+length pairs.

---

## 3. Iteration

**C**
```c
for (int i = 0; i < n; i++) {
    printf("%d\n", arr[i]);
}

for (int i = n - 1; i >= 0; i--) {
    printf("%d\n", arr[i]);
}
```

**Rust**
```rust
for x in &v {
    println!("{x}");
}

for i in 0..n {
    println!("{}", v[i]);
}

for (i, x) in v.iter().enumerate() {
    println!("{i}: {x}");
}

for x in v.iter().rev() {
    println!("{x}");
}

for i in (0..n).rev() {
    println!("{}", v[i]);
}

for pair in v.windows(2) {
    let (a, b) = (pair[0], pair[1]);
}
```

**Key difference:** Rust iterators replace index-based loops; `enumerate()` gives you the index when you need it, `rev()` reverses direction.

---

## 4. Strings

**C**
```c
char s[] = "hello";
int len = strlen(s);
char c = s[0];

for (int i = 0; i < len; i++) {
    char c = s[i];
}

char buf[100];
sprintf(buf, "%s world", s);
```

**Rust**
```rust
let s = String::from("hello");       // owned, heap-allocated
let s_ref: &str = &s;                // borrowed view
let len = s.len();                    // byte length
let char_count = s.chars().count();   // character count (Unicode-safe)

for c in s.chars() {
    // c is a char (Unicode scalar)
}

for (i, c) in s.chars().enumerate() {
    // index + character
}

let bytes = s.as_bytes();            // &[u8], for ASCII-only work
let c = bytes[0] as char;            // index into bytes, not chars

let combined = format!("{s} world");

let v: Vec<char> = s.chars().collect();       // when you need indexing by char
let back: String = v.iter().collect();        // char vec back to String

s.contains("ell");
s.starts_with("he");
let trimmed = s.trim();
let parts: Vec<&str> = s.split(',').collect();

let sub = &s[0..3];                  // byte-index slice: "hel"
```

**Key difference:** Rust strings are UTF-8 encoded; you cannot index by character position directly -- use `.chars()` or convert to `Vec<char>` when you need random access.

---

## 5. HashMap & HashSet

**C**
```c
// No standard hash map. Typically hand-rolled or use uthash.
```

**Rust**
```rust
use std::collections::{HashMap, HashSet};

let mut map: HashMap<i32, i32> = HashMap::new();
map.insert(1, 100);

if let Some(val) = map.get(&1) {
    println!("{val}");
}

let val = map.get(&1).copied().unwrap_or(0);

map.entry(1).or_insert(0);

*map.entry(key).or_insert(0) += 1;    // frequency counting pattern

map.contains_key(&1);
map.remove(&1);

for (k, v) in &map {
    println!("{k}: {v}");
}

let mut set: HashSet<i32> = HashSet::new();
set.insert(1);
set.contains(&1);
set.remove(&1);

let set: HashSet<i32> = vec![1, 2, 3].into_iter().collect();
```

**Key difference:** `entry()` API is the idiomatic way to do insert-or-update; the `*map.entry(k).or_insert(0) += 1` pattern replaces the classic "check-then-insert" two-step.

---

## 6. Sorting

**C**
```c
int cmp(const void *a, const void *b) {
    return (*(int *)a - *(int *)b);
}
qsort(arr, n, sizeof(int), cmp);
```

**Rust**
```rust
let mut v = vec![3, 1, 2];
v.sort();                                 // ascending, requires Ord
v.sort_unstable();                        // faster, no stability guarantee

v.sort_by(|a, b| b.cmp(a));              // descending

v.sort_by_key(|x| x.abs());              // sort by derived key

let mut intervals = vec![(1,3), (0,2), (2,4)];
intervals.sort_by_key(|&(start, _)| start);

intervals.sort_by(|a, b| {
    a.0.cmp(&b.0).then(a.1.cmp(&b.1))    // multi-key sort
});

let mut floats = vec![1.5, 0.3, 2.1];
floats.sort_by(|a, b| a.partial_cmp(b).unwrap());  // f64 is not Ord
```

**Key difference:** Rust sort is in-place on `&mut Vec` or `&mut [T]`; floating-point types need `partial_cmp` because `NaN` breaks total ordering.

---

## 7. Stack & Queue

**C**
```c
// Stack via array
int stack[100], top = -1;
stack[++top] = val;        // push
int val = stack[top--];    // pop

// Queue via circular buffer or linked list -- manual work
```

**Rust**
```rust
use std::collections::VecDeque;

let mut stack: Vec<i32> = Vec::new();
stack.push(1);
let top = stack.pop();             // Option<i32>
let peek = stack.last();           // Option<&i32>

let mut queue: VecDeque<i32> = VecDeque::new();
queue.push_back(1);                // enqueue
let front = queue.pop_front();     // dequeue, Option<i32>
let peek = queue.front();          // Option<&i32>

// Monotonic stack pattern
let mut mono: Vec<(usize, i32)> = Vec::new();
for (i, &val) in nums.iter().enumerate() {
    while let Some(&(_, top_val)) = mono.last() {
        if top_val <= val {
            mono.pop();
        } else {
            break;
        }
    }
    mono.push((i, val));
}
```

**Key difference:** `Vec` is a natural stack (`push`/`pop` operate on the back); `VecDeque` gives O(1) operations on both ends.

---

## 8. Pattern Matching

**C**
```c
switch (x) {
    case 1: do_one(); break;
    case 2: do_two(); break;
    default: do_other(); break;
}

if (ptr == NULL) { /* handle */ }
```

**Rust**
```rust
match x {
    1 => do_one(),
    2 => do_two(),
    3 | 4 => do_three_or_four(),
    5..=10 => do_range(),
    _ => do_other(),
}

match (a, b) {
    (0, 0) => "origin",
    (x, 0) | (0, x) => "on axis",
    (x, y) if x == y => "diagonal",
    _ => "other",
}

match some_option {
    Some(val) => use_val(val),
    None => handle_missing(),
}

if let Some(val) = some_option {
    use_val(val);
}

while let Some(top) = stack.pop() {
    process(top);
}

let result = match direction {
    "left" => -1,
    "right" => 1,
    _ => 0,
};
```

**Key difference:** `match` is exhaustive (the compiler forces you to handle every case) and can destructure tuples, enums, and nested structures.

---

## 9. Option & Result

**C**
```c
int *find(int *arr, int n, int target) {
    for (int i = 0; i < n; i++)
        if (arr[i] == target) return &arr[i];
    return NULL;
}

if (result == NULL) { /* not found */ }
```

**Rust**
```rust
fn find(v: &[i32], target: i32) -> Option<usize> {
    v.iter().position(|&x| x == target)
}

let idx = find(&v, 5).unwrap();            // panics if None
let idx = find(&v, 5).unwrap_or(0);        // default value
let idx = find(&v, 5)?;                    // propagate None to caller

let val = some_opt.map(|x| x * 2);         // transform inner value
let val = some_opt.unwrap_or_else(|| compute_default());
let val = some_opt.filter(|&x| x > 0);

if let Some(x) = opt {
    // use x
}

let a = opt_a.or(opt_b);                   // first Some wins
let a = opt_a.and_then(|x| further(x));    // chain computations
```

**Key difference:** `Option<T>` makes the absence of a value explicit in the type system; `?` operator short-circuits and propagates `None` (or `Err`) to the caller.

---

## 10. Common Iterator Chains

**C**
```c
// Sum of squares of even numbers -- manual loop
int sum = 0;
for (int i = 0; i < n; i++) {
    if (arr[i] % 2 == 0)
        sum += arr[i] * arr[i];
}
```

**Rust**
```rust
let sum: i32 = v.iter()
    .filter(|&&x| x % 2 == 0)
    .map(|&x| x * x)
    .sum();

let max = v.iter().max().copied();               // Option<i32>
let min = v.iter().min().copied();

let total: i32 = v.iter().sum();

let product: i32 = v.iter().product();

let all_positive = v.iter().all(|&x| x > 0);
let any_negative = v.iter().any(|&x| x < 0);

let first_even = v.iter().find(|&&x| x % 2 == 0);
let pos = v.iter().position(|&x| x == target);

let doubled: Vec<i32> = v.iter().map(|&x| x * 2).collect();

let flat: Vec<i32> = matrix.iter().flatten().copied().collect();

let s: String = chars.iter().collect();

let pairs: Vec<_> = a.iter().zip(b.iter()).collect();

for w in v.windows(3) {
    // sliding window of size 3
}

for chunk in v.chunks(2) {
    // non-overlapping chunks of size 2
}

let prefix_sums: Vec<i32> = v.iter()
    .scan(0, |acc, &x| { *acc += x; Some(*acc) })
    .collect();

let count = v.iter().filter(|&&x| x > 0).count();

let freq: HashMap<i32, usize> = v.iter()
    .fold(HashMap::new(), |mut acc, &x| {
        *acc.entry(x).or_insert(0) += 1;
        acc
    });
```

**Key difference:** Iterator chains are lazy (nothing executes until a terminal like `collect()`, `sum()`, or `count()` is called) and the compiler optimizes them into tight loops comparable to hand-written C.

---

## 11. Ownership Quick Reference

**C**
```c
int *p = malloc(sizeof(int));
int *q = p;       // two pointers to same memory -- your problem now
free(p);           // q is now dangling
```

**Rust**
```rust
let s = String::from("hello");
let s2 = s;                    // MOVE: s is now invalid
// println!("{s}");            // compile error

let s = String::from("hello");
let s2 = s.clone();            // CLONE: deep copy, both valid

fn process(v: &Vec<i32>) {}    // BORROW: read-only reference
fn modify(v: &mut Vec<i32>) {} // MUTABLE BORROW: exclusive write access

process(&v);                   // v is still valid after call
modify(&mut v);                // v is still valid after call
```

**When to use what:**

| Situation | Use |
|---|---|
| Function only reads data | `&T` (shared borrow) |
| Function modifies data | `&mut T` (exclusive borrow) |
| Function needs to own (store, return, send to thread) | `T` (move) |
| Need two owners of the same data | `.clone()` |
| i32, f64, bool, char, tuples of these | Copy -- they never move |

```rust
fn two_sum(nums: &[i32], target: i32) -> Vec<i32> {
    // &[i32] borrows the input, Vec<i32> owns the output
    let mut map = HashMap::new();
    for (i, &num) in nums.iter().enumerate() {
        if let Some(&j) = map.get(&(target - num)) {
            return vec![j as i32, i as i32];
        }
        map.insert(num, i);
    }
    vec![]
}
```

**Key difference:** Rust enforces at compile time that you have either one mutable reference OR any number of shared references, never both -- this eliminates data races and dangling pointers.

---

## 12. Linked List Patterns

**C**
```c
struct ListNode {
    int val;
    struct ListNode *next;
};

struct ListNode *reverse(struct ListNode *head) {
    struct ListNode *prev = NULL, *curr = head;
    while (curr) {
        struct ListNode *next = curr->next;
        curr->next = prev;
        prev = curr;
        curr = next;
    }
    return prev;
}
```

**Rust**
```rust
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    fn new(val: i32) -> Self {
        ListNode { val, next: None }
    }
}

type List = Option<Box<ListNode>>;

fn reverse(head: List) -> List {
    let mut prev: List = None;
    let mut curr = head;
    while let Some(mut node) = curr {
        curr = node.next.take();    // take() replaces with None and returns old value
        node.next = prev;
        prev = Some(node);
    }
    prev
}

fn get_length(head: &List) -> i32 {
    let mut count = 0;
    let mut curr = head;
    while let Some(node) = curr {
        count += 1;
        curr = &node.next;
    }
    count
}

fn from_vec(v: Vec<i32>) -> List {
    let mut head: List = None;
    for &val in v.iter().rev() {
        let mut node = Box::new(ListNode::new(val));
        node.next = head;
        head = Some(node);
    }
    head
}

fn merge_two(l1: List, l2: List) -> List {
    match (l1, l2) {
        (None, None) => None,
        (Some(n), None) | (None, Some(n)) => Some(n),
        (Some(mut n1), Some(mut n2)) => {
            if n1.val <= n2.val {
                n1.next = merge_two(n1.next, Some(n2));
                Some(n1)
            } else {
                n2.next = merge_two(Some(n1), n2.next);
                Some(n2)
            }
        }
    }
}
```

**Key difference:** `Option<Box<ListNode>>` encodes nullability in the type; `.take()` is the primary tool for moving nodes around without fighting the borrow checker.

---

## 13. Tree Patterns

**C**
```c
struct TreeNode {
    int val;
    struct TreeNode *left;
    struct TreeNode *right;
};

void inorder(struct TreeNode *root) {
    if (!root) return;
    inorder(root->left);
    printf("%d ", root->val);
    inorder(root->right);
}
```

**Rust**
```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Rc<RefCell<TreeNode>>>,
    pub right: Option<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    fn new(val: i32) -> Self {
        TreeNode { val, left: None, right: None }
    }
}

type Tree = Option<Rc<RefCell<TreeNode>>>;

fn inorder(root: &Tree) -> Vec<i32> {
    let mut result = Vec::new();
    fn dfs(node: &Tree, out: &mut Vec<i32>) {
        if let Some(n) = node {
            let n = n.borrow();
            dfs(&n.left, out);
            out.push(n.val);
            dfs(&n.right, out);
        }
    }
    dfs(root, &mut result);
    result
}

fn max_depth(root: &Tree) -> i32 {
    match root {
        None => 0,
        Some(node) => {
            let node = node.borrow();
            1 + max_depth(&node.left).max(max_depth(&node.right))
        }
    }
}

fn bfs_level_order(root: Tree) -> Vec<Vec<i32>> {
    use std::collections::VecDeque;
    let mut result = Vec::new();
    let mut queue = VecDeque::new();
    if let Some(r) = root {
        queue.push_back(r);
    }
    while !queue.is_empty() {
        let mut level = Vec::new();
        for _ in 0..queue.len() {
            let node = queue.pop_front().unwrap();
            let node = node.borrow();
            level.push(node.val);
            if let Some(left) = &node.left {
                queue.push_back(Rc::clone(left));
            }
            if let Some(right) = &node.right {
                queue.push_back(Rc::clone(right));
            }
        }
        result.push(level);
    }
    result
}

fn is_valid_bst(root: &Tree) -> bool {
    fn check(node: &Tree, min: i64, max: i64) -> bool {
        match node {
            None => true,
            Some(n) => {
                let n = n.borrow();
                let v = n.val as i64;
                v > min && v < max
                    && check(&n.left, min, v)
                    && check(&n.right, v, max)
            }
        }
    }
    check(root, i64::MIN, i64::MAX)
}
```

**Key difference:** `Rc<RefCell<T>>` gives shared ownership with interior mutability -- `Rc` handles multiple references (parent/child both accessible), `RefCell` moves borrow checking to runtime. Use `Rc::clone()` to create a new reference (cheap, just increments a counter), and `.borrow()` / `.borrow_mut()` to access the inner node.
