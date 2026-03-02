#[macro_export]
macro_rules! list {
    () => { None };
    ($($val:expr),+ $(,)?) => {
        $crate::types::list_from_vec(vec![$($val),+])
    };
}

#[macro_export]
macro_rules! tree {
    (@val null) => { None };
    (@val $val:expr) => { Some($val) };
    () => { None };
    ($($val:tt),+ $(,)?) => {{
        let values: Vec<Option<i32>> = vec![$($crate::tree!(@val $val)),+];
        $crate::types::build_tree(&values)
    }};
}

#[cfg(test)]
mod tests {
    use crate::types::*;

    #[test]
    fn list_macro_empty() {
        let l: Option<Box<ListNode>> = list![];
        assert_eq!(l, None);
    }

    #[test]
    fn list_macro_values() {
        assert_eq!(list_to_vec(list![1, 2, 3]), vec![1, 2, 3]);
    }

    #[test]
    fn tree_macro_empty() {
        let t: Option<std::rc::Rc<std::cell::RefCell<TreeNode>>> = tree![];
        assert_eq!(t, None);
    }

    #[test]
    fn tree_macro_single() {
        assert_eq!(tree_to_vec(tree![1]), vec![Some(1)]);
    }

    #[test]
    fn tree_macro_with_nulls() {
        assert_eq!(
            tree_to_vec(tree![3, 9, 20, null, null, 15, 7]),
            vec![Some(3), Some(9), Some(20), None, None, Some(15), Some(7)]
        );
    }
}
