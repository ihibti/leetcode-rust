//! Problem: merge-two-lists
//! Difficulty: easy
//! Tags: linked-list
//! Rust concepts: ownership, Option-Box, pattern-matching, mem-swap
//! Date: 2026-03-14
//! Time: 35m

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        ListNode { next: None, val }
    }
}

pub struct Solution;

impl Solution {
    pub fn merge_two_lists(
        mut list1: Option<Box<ListNode>>,
        mut list2: Option<Box<ListNode>>,
    ) -> Option<Box<ListNode>> {
        let mut r = &mut list1;
        while list2.is_some() {
            if r.is_none() || list2.as_ref().unwrap().val < r.as_ref().unwrap().val {
                std::mem::swap(r, &mut list2);
            }
            r = &mut r.as_mut().unwrap().next;
        }
        list1
    }
}
