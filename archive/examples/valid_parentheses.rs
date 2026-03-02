//! Problem: valid-parentheses
//! Difficulty: easy
//! Tags: string, stack
//! Rust concepts: pattern-matching, Vec-as-stack, chars
//! Date: 2026-03-14
//! Time: 10m

pub struct Solution;

impl Solution {
    pub fn is_valid(s: String) -> bool {
        let mut stack = Vec::new();
        for ch in s.chars() {
            match ch {
                '(' | '[' | '{' => stack.push(ch),
                ')' => if stack.pop() != Some('(') { return false; },
                ']' => if stack.pop() != Some('[') { return false; },
                '}' => if stack.pop() != Some('{') { return false; },
                _ => {}
            }
        }
        stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid() {
        assert!(Solution::is_valid("()[]{}".into()));
    }

    #[test]
    fn nested() {
        assert!(Solution::is_valid("{[()]}".into()));
    }

    #[test]
    fn invalid() {
        assert!(!Solution::is_valid("(]".into()));
    }
}
