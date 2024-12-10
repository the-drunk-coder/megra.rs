use std::{collections::HashMap, fmt::Debug, hash::Hash};

/**
 * basically a suffix tree to find an appropriate duration, especially
 * when inferring from rules ...
 **/

pub struct DurationTreeNode<T: Eq + Hash + Copy + Debug> {
    pub label: Vec<T>,
    pub children: HashMap<T, DurationTreeNode<T>>,
    pub duration: Option<usize>, // durations in milliseconds
}

impl<T: Eq + Hash + Copy + Debug> DurationTreeNode<T> {
    fn new(label: &[T], duration: Option<usize>) -> Self {
        DurationTreeNode {
            label: label.to_vec(),
            children: HashMap::new(),
            duration,
        }
    }

    fn get_or_insert_child(
        &mut self,
        key: T,
        label: &[T],
        duration: Option<usize>,
    ) -> &mut DurationTreeNode<T> {
        self.children
            .entry(key)
            .or_insert(DurationTreeNode::new(label, duration))
    }
}

/// Add a leaf to a Probabilistic suffix tree, add nodes along the path if necessary.
pub fn add_leaf<T: Eq + Copy + Hash + Debug>(
    root: &mut DurationTreeNode<T>,
    label: &[T],
    duration: Option<usize>,
) {
    if !label.is_empty() {
        add_leaf_recursion(root, label, label.len() - 1, duration);
    }
}

/// Add a node to a Probabilistic Suffix Tree node, adding nodes on the path if necessary.
/// If specified, copy gamma function of parent node.
fn add_leaf_recursion<T: Eq + Copy + Hash + Debug>(
    node: &mut DurationTreeNode<T>,
    label: &[T],
    label_idx: usize,
    duration: Option<usize>,
) {
    //println!("LAB ID {label_idx} {label:?}");
    let path_node = node.get_or_insert_child(label[label_idx], &label[label_idx..], None);

    if label_idx != 0 {
        add_leaf_recursion(path_node, label, label_idx - 1, duration);
    } else {
        path_node.duration = duration;
    }
}

pub fn find_longest_suffix_duration<'a, T: Eq + Copy + Hash + Debug>(
    root: &'a DurationTreeNode<T>,
    label: &[T],
    last_duration: Option<usize>,
) -> Option<usize> {
    if label.is_empty() {
        // found a leaf
        if root.duration.is_some() {
            root.duration
        } else {
            last_duration
        }
    } else {
        let last = label.last().unwrap();
        if root.children.contains_key(last) {
            find_longest_suffix_duration(
                root.children.get(last).unwrap(),
                &label[..(label.len() - 1)],
                if root.duration.is_some() {
                    root.duration
                } else {
                    last_duration
                },
            )
        } else {
            if root.duration.is_some() {
                root.duration
            } else {
                last_duration
            }
        }
    }
}

/// Find the longest suffix for label plus a symbol in the tree.
pub fn find_longest_suffix_duration_with_symbol<'a, T: Eq + Copy + Hash + Debug>(
    root: &'a DurationTreeNode<T>,
    label: &[T],
    symbol: &T,
) -> Option<usize> {
    if root.children.contains_key(symbol) {
        find_longest_suffix_duration(root.children.get(symbol).unwrap(), label, None)
    } else {
        root.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_detection() {
        let label_1 = vec!['a', 'b'];
        let label_2 = vec!['a', 'c'];
        let label_3 = vec!['a', 'a', 'a', 'b'];

        let mut root = DurationTreeNode::<char>::new(&vec![], None);

        add_leaf(&mut root, &label_1, Some(100));
        add_leaf(&mut root, &label_2, Some(200));
        add_leaf(&mut root, &label_3, Some(300));

        assert_eq!(
            find_longest_suffix_duration(&root, &vec!['a', 'b'], None),
            Some(100)
        );
        assert_eq!(
            find_longest_suffix_duration(&root, &vec!['a', 'a', 'b'], None),
            Some(100)
        );
        assert_eq!(
            find_longest_suffix_duration(&root, &vec!['a', 'a', 'a', 'b'], None),
            Some(300)
        );

        assert_eq!(
            find_longest_suffix_duration(&root, &vec!['a', 'c'], None),
            Some(200)
        );
        assert_eq!(
            find_longest_suffix_duration(&root, &vec!['a', 'a', 'c'], None),
            Some(200)
        );
    }
}
