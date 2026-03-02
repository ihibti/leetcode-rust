use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

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

pub fn list_from_vec(v: Vec<i32>) -> Option<Box<ListNode>> {
    let mut head = None;
    for &val in v.iter().rev() {
        let mut node = ListNode::new(val);
        node.next = head;
        head = Some(Box::new(node));
    }
    head
}

pub fn list_to_vec(mut head: Option<Box<ListNode>>) -> Vec<i32> {
    let mut result = vec![];
    while let Some(node) = head {
        result.push(node.val);
        head = node.next;
    }
    result
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Rc<RefCell<TreeNode>>>,
    pub right: Option<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        TreeNode {
            val,
            left: None,
            right: None,
        }
    }
}

pub fn build_tree(values: &[Option<i32>]) -> Option<Rc<RefCell<TreeNode>>> {
    if values.is_empty() || values[0].is_none() {
        return None;
    }
    let root = Rc::new(RefCell::new(TreeNode::new(values[0].unwrap())));
    let mut queue = VecDeque::new();
    queue.push_back(Rc::clone(&root));
    let mut i = 1;
    while i < values.len() {
        if let Some(node) = queue.pop_front() {
            if i < values.len() {
                if let Some(val) = values[i] {
                    let left = Rc::new(RefCell::new(TreeNode::new(val)));
                    node.borrow_mut().left = Some(Rc::clone(&left));
                    queue.push_back(left);
                }
                i += 1;
            }
            if i < values.len() {
                if let Some(val) = values[i] {
                    let right = Rc::new(RefCell::new(TreeNode::new(val)));
                    node.borrow_mut().right = Some(Rc::clone(&right));
                    queue.push_back(right);
                }
                i += 1;
            }
        }
    }
    Some(root)
}

pub fn tree_to_vec(root: Option<Rc<RefCell<TreeNode>>>) -> Vec<Option<i32>> {
    let mut result = vec![];
    let mut queue = VecDeque::new();
    queue.push_back(root);
    while let Some(node_opt) = queue.pop_front() {
        match node_opt {
            Some(node) => {
                let borrowed = node.borrow();
                result.push(Some(borrowed.val));
                queue.push_back(borrowed.left.clone());
                queue.push_back(borrowed.right.clone());
            }
            None => {
                result.push(None);
            }
        }
    }
    while result.last() == Some(&None) {
        result.pop();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_empty() {
        assert_eq!(list_from_vec(vec![]), None);
    }

    #[test]
    fn list_single() {
        let list = list_from_vec(vec![1]);
        assert_eq!(list_to_vec(list), vec![1]);
    }

    #[test]
    fn list_multiple() {
        let list = list_from_vec(vec![1, 2, 3]);
        assert_eq!(list_to_vec(list), vec![1, 2, 3]);
    }

    #[test]
    fn tree_empty() {
        assert_eq!(build_tree(&[]), None);
    }

    #[test]
    fn tree_single() {
        let tree = build_tree(&[Some(1)]);
        assert_eq!(tree_to_vec(tree), vec![Some(1)]);
    }

    #[test]
    fn tree_with_nulls() {
        let tree = build_tree(&[Some(3), Some(9), Some(20), None, None, Some(15), Some(7)]);
        assert_eq!(
            tree_to_vec(tree),
            vec![Some(3), Some(9), Some(20), None, None, Some(15), Some(7)]
        );
    }
}
