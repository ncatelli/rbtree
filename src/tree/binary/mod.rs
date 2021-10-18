//! Provides an implementation of a Binary Tree.

extern crate alloc;
use super::{Direction, Directional, Node, NodeRef};
use alloc::{boxed::Box, vec::Vec};

/// Represents the three possible situations that a node can encounter on a delete,
#[derive(Clone, Copy, PartialEq)]
enum DeleteSuccessor<T> {
    /// Node has two children. Return the in-order successor.
    Double(Option<NodeRef<T>>),
    /// Node has a single child.
    Single(NodeRef<T>),
    /// Node has no children (is a leaf or root).
    /// Can be deleted directly.
    None,
}

/// SearchResult represents the results of a binary tree search.
#[derive(Debug)]
enum SearchResult<T> {
    /// Hit signifies the exact value was found in the tree and
    /// contains a reference to the NodeId for said value.
    Hit(NodeRef<T>),
    /// Miss represents the value was not found in the tree and represents the
    /// nearest parent node.
    Miss(NodeRef<T>),
    /// Empty represents an empty tree.
    Empty,
}

impl<T> SearchResult<T> {
    /// Calls `f` if the self is `SearchResult::Hit` returning the result of
    /// `f` wrapped in `Some` otherwise `None` is returned.
    fn hit_then<F, B>(self, f: F) -> Option<B>
    where
        F: Fn(NodeRef<T>) -> B,
    {
        match self {
            SearchResult::Hit(node) => Some(f(node)),
            _ => None,
        }
    }
}

/// An implementation of a Binary Tree
#[derive(Debug)]
pub struct BinaryTree<T>
where
    T: PartialEq + PartialOrd,
{
    root: Option<NodeRef<T>>,
}

impl<T> BinaryTree<T>
where
    T: PartialEq + PartialOrd,
{
    /// Instantiates a new Binary tree from an initial value.
    pub fn new(root: T) -> Self {
        let node = Node::new(root, None, None, None);
        let root_ptr = NodeRef::from(node);

        Self {
            root: Some(root_ptr),
        }
    }
}

// helper methods
impl<T> BinaryTree<T>
where
    T: PartialEq + PartialOrd,
{
    /// Returns a boolean representing if the tree is empty or not.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Searches for a value in the tree returning a SearchResult that
    /// captures if the search yield a hit, miss or empty tree.  
    unsafe fn find_nearest_node(&self, value: &T) -> SearchResult<T> {
        if let Some(root) = self.root {
            let mut next_step = root;
            loop {
                if value == &next_step.as_ref().inner {
                    return SearchResult::Hit(next_step);
                } else if value <= &next_step.as_ref().inner {
                    // if left leaf exists follow that direction.
                    match &next_step.as_ref().left {
                        Some(left) => next_step = *left,
                        // return the parent
                        None => return SearchResult::Miss(next_step),
                    }
                } else {
                    // if right leaf exists follow that direction.
                    match &next_step.as_ref().right {
                        Some(right) => next_step = *right,
                        // return the parent
                        None => return SearchResult::Miss(next_step),
                    }
                }
            }
        } else {
            SearchResult::Empty
        }
    }

    /// Inserts a value `T` into the tree returning a the modified tree in
    /// place.
    pub fn insert(mut self, value: T) -> Self {
        self.insert_mut(value);
        self
    }

    /// Inserts a value `T` into the tree. If the value already exists in the
    /// tree, nothing is done.
    pub fn insert_mut(&mut self, value: T) {
        unsafe { self.insert_mut_unchecked(value) }
    }

    unsafe fn insert_mut_unchecked(&mut self, value: T) {
        let nearest = self.find_nearest_node(&value);
        match nearest {
            SearchResult::Hit(_) => (),
            SearchResult::Empty => {
                let node = Node::new(value, None, None, None);
                self.root = Some(NodeRef::from(node));
            }
            SearchResult::Miss(mut parent_node) => {
                let is_left = value < parent_node.as_ref().inner;
                let child = Node::new(value, Some(parent_node), None, None);
                let child_ptr = NodeRef::from(child);
                if is_left {
                    parent_node.as_mut().left = Some(child_ptr);
                } else {
                    parent_node.as_mut().right = Some(child_ptr);
                }
            }
        };
    }

    /// Remove a value, `T`, from the tree, returning the modifed tree.
    pub fn remove(mut self, value: &T) -> Self {
        self.remove_mut(value);
        self
    }

    /// Remove a value, `T`, from the tree in place.
    pub fn remove_mut(&mut self, value: &T) -> Option<T> {
        unsafe { self.remove_mut_unchecked(value) }
    }

    unsafe fn remove_mut_unchecked(&mut self, value: &T) -> Option<T> {
        let node_to_be_deleted = self.find_nearest_node(value).hit_then(|node| node)?;
        let optional_node_direction = node_to_be_deleted.as_ref().direction();
        let optional_parent = node_to_be_deleted.as_ref().parent;
        let optional_left_child = node_to_be_deleted.as_ref().left;
        let optional_right_child = node_to_be_deleted.as_ref().right;

        let delete_successor = match (optional_left_child, optional_right_child) {
            (None, None) => DeleteSuccessor::None,
            (Some(successor), None) | (None, Some(successor)) => DeleteSuccessor::Single(successor),
            (Some(_), Some(_)) => {
                DeleteSuccessor::Double(self.find_in_order_successor(node_to_be_deleted))
            }
        };

        match delete_successor {
            // can be directly deleted
            DeleteSuccessor::None => {
                // convert to a box so it is dropped
                let boxed_node_to_be_deleted = Box::from_raw(node_to_be_deleted.as_ptr());
                if let Some(direction) = optional_node_direction {
                    // if it has a direction it's safe to unwrap.
                    let mut parent = optional_parent.expect("unable to unwrap parent");
                    match direction {
                        Direction::Left => parent.as_mut().left = None,
                        Direction::Right => parent.as_mut().right = None,
                    };
                } else {
                    // Mark the tree as empty if this is the last node.
                    self.root = None;
                }

                // Take ownership of the inner value
                let inner = boxed_node_to_be_deleted.inner;
                Some(inner)
            }
            DeleteSuccessor::Single(mut x) => {
                // convert to a box so it is dropped
                let boxed_node_to_be_deleted = Box::from_raw(node_to_be_deleted.as_ptr());

                if let Some(direction) = optional_node_direction {
                    // if it has a direction it's safe to unwrap.
                    let mut parent = optional_parent.expect("unable to unwrap parent");
                    match direction {
                        Direction::Left => parent.as_mut().left = Some(x),
                        Direction::Right => parent.as_mut().right = Some(x),
                    };
                } else {
                    self.root = Some(x);
                }

                x.as_mut().parent = boxed_node_to_be_deleted.parent;

                // Take ownership of the inner value
                let inner = boxed_node_to_be_deleted.inner;
                Some(inner)
            }
            DeleteSuccessor::Double(in_order_successor) => {
                // convert to a box so it is dropped
                let boxed_node_to_be_deleted = Box::from_raw(node_to_be_deleted.as_ptr());
                let mut y =
                    in_order_successor.expect("in order successor is null on a two child delete");
                let y_direction = y.as_ref().direction().expect("y has no parent");

                let x = y.as_ref().right;

                // If y is not a child of nodeToBeDeletedtransplant y with rightChild of y
                if y.as_ref().parent != Some(node_to_be_deleted) {
                    // safe to unwrap, y is guaranteed a parent by the sucessor check.
                    let mut y_parent = y.as_ref().parent.expect("y has no parent");

                    match y_direction {
                        Direction::Left => y_parent.as_mut().left = x,
                        Direction::Right => y_parent.as_mut().right = x,
                    }
                }

                // Transplant nodeToBeDeleted with y.
                y.as_mut().parent = boxed_node_to_be_deleted.parent;
                match boxed_node_to_be_deleted.direction() {
                    // safe to unwrap parents because of direction check
                    Some(Direction::Left) => {
                        boxed_node_to_be_deleted.parent.unwrap().as_mut().left = Some(y)
                    }
                    Some(Direction::Right) => {
                        boxed_node_to_be_deleted.parent.unwrap().as_mut().right = Some(y)
                    }
                    None => self.root = Some(y),
                };

                y.as_mut().left = boxed_node_to_be_deleted.left;
                if let Some(mut left) = boxed_node_to_be_deleted.left {
                    left.as_mut().parent = Some(y);
                }

                Some(boxed_node_to_be_deleted.inner)
            }
        }
    }

    unsafe fn find_in_order_successor(&self, node: NodeRef<T>) -> Option<NodeRef<T>> {
        let optional_right_child = node.as_ref().right;

        optional_right_child.and_then(|child| self.find_min_from(child))
    }

    /// Returns the node with the left-most value (smallest) or `None` if the
    /// tree is empty.
    pub fn min(&self) -> Option<&T> {
        unsafe {
            self.root
                .and_then(|base_node| self.find_min_from(base_node))
                .map(|node| &(*node.as_ptr()).inner)
        }
    }

    /// Returns the node with the left-most value (smallest) or `None`, if
    /// empty, starting from a given base node.
    unsafe fn find_min_from(&self, base: NodeRef<T>) -> Option<NodeRef<T>> {
        let mut current = Some(base);
        let mut left_most_node = current;
        while let Some(id) = current {
            left_most_node = current;
            current = id.as_ref().left;
        }
        left_most_node
    }

    /// Returns the node with the right-most value (largest) or `None` if the
    /// tree is empty.
    pub fn max(&self) -> Option<&T> {
        unsafe {
            self.root
                .and_then(|base_node| self.find_max_from(base_node))
                .map(|node| &(*node.as_ptr()).inner)
        }
    }

    /// Returns the node with the right-most value (largest) or `None`, if
    /// empty, starting from a given base node.
    unsafe fn find_max_from(&self, base_node_id: NodeRef<T>) -> Option<NodeRef<T>> {
        let mut current = Some(base_node_id);
        let mut right_most_node = current;
        while let Some(id) = current {
            right_most_node = current;
            current = id.as_ref().right;
        }
        right_most_node
    }

    /// Returns an Iterator for traversing an array in order.
    pub fn traverse_in_order(&self) -> IterInOrder<'_, T> {
        IterInOrder::new(self)
    }
}

impl<T> Drop for BinaryTree<T>
where
    T: PartialOrd + PartialEq,
{
    fn drop(&mut self) {
        unsafe {
            let mut next = self.min();
            while let Some(value) = next {
                let node = self.find_nearest_node(value).hit_then(|node| node).unwrap();
                let inner_val = &node.as_ptr().as_ref().unwrap().inner;
                self.remove_mut(inner_val);

                next = self.min();
            }

            self.root = None;
        }
    }
}

impl<T> Default for BinaryTree<T>
where
    T: PartialEq + PartialOrd,
{
    fn default() -> Self {
        Self { root: None }
    }
}

pub struct IterInOrder<'a, T>
where
    T: PartialEq + PartialOrd + 'a,
{
    inner: core::marker::PhantomData<&'a BinaryTree<T>>,
    left_most_node: Option<NodeRef<T>>,
    stack: Vec<NodeRef<T>>,
}

impl<'a, T: 'a> IterInOrder<'a, T>
where
    T: PartialEq + PartialOrd + 'a,
{
    pub fn new(inner: &'a BinaryTree<T>) -> Self {
        Self {
            inner: core::marker::PhantomData,
            left_most_node: inner.root,
            stack: Vec::new(),
        }
    }
}

impl<'a, V: 'a> Iterator for IterInOrder<'a, V>
where
    V: PartialEq + PartialOrd + 'a,
{
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(id) = self.left_most_node {
            self.stack.push(id);

            self.left_most_node = id.as_ref().left;
        }
        if let Some(up_from_current) = self.stack.pop() {
            self.left_most_node = up_from_current.as_ref().right;

            Some(unsafe { &(*up_from_current.as_ptr()).inner })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate alloc;
    use alloc::vec;

    #[test]
    fn should_return_correct_empty_state_when_tree_has_values() {
        let tree = BinaryTree::<usize>::default();

        assert!(tree.is_empty());
        assert!(!tree.insert(5).is_empty());
    }

    #[test]
    fn should_yield_correct_min_and_max_for_a_given_tree() {
        let tree = vec![10, 5, 15, 25, 20]
            .into_iter()
            .fold(BinaryTree::default(), |tree, x| tree.insert(x));

        assert_eq!(Some(&25), tree.max());
        assert_eq!(Some(&5), tree.min());

        let empty_tree = BinaryTree::<usize>::default();

        assert_eq!(None, empty_tree.max());
        assert_eq!(None, empty_tree.min());
    }

    #[test]
    fn should_traverse_in_order() {
        let tree = vec![10, 5, 15, 25, 20]
            .into_iter()
            .fold(BinaryTree::default(), |tree, x| tree.insert(x));

        let mut i = tree.traverse_in_order();

        assert_eq!(Some(&5), i.next());
        assert_eq!(Some(&10), i.next());
        assert_eq!(Some(&15), i.next());
        assert_eq!(Some(&20), i.next());
        assert_eq!(Some(&25), i.next());
        assert_eq!(None, i.next());

        let tree = (0..1024)
            .rev()
            .fold(BinaryTree::default(), |tree, x| tree.insert(x));

        let received: Vec<u16> = tree.traverse_in_order().copied().collect();
        let expected: Vec<u16> = (0..1024).collect();
        assert_eq!(expected, received);
    }

    #[test]
    fn should_remove_node_with_no_children() {
        let node_values = [10, 5, 1, 15];
        let tree = node_values
            .to_vec()
            .into_iter()
            .fold(BinaryTree::default(), |tree, x| tree.insert(x))
            .remove(&1);

        let left_child_of_root = unsafe { tree.find_nearest_node(&5).hit_then(|node| node) };

        assert_eq!(None, left_child_of_root.and_then(|c| c.as_ref().left));
    }

    #[test]
    fn should_remove_node_with_one_child_while_retaining_relationships() {
        let node_values = [10, 5, 1, 15];
        let tree = node_values
            .to_vec()
            .into_iter()
            .fold(BinaryTree::default(), |tree, x| tree.insert(x))
            .remove(&10);

        let root = unsafe { tree.find_nearest_node(&15).hit_then(|node| node) };
        let left_child = unsafe { tree.find_nearest_node(&5).hit_then(|node| node) };
        let left_sub_child = unsafe { tree.find_nearest_node(&1).hit_then(|node| node) };

        assert_eq!(root, tree.root);

        assert_eq!(root.and_then(|r| r.as_ref().left), left_child);
        assert_eq!(root, left_child.and_then(|c| c.as_ref().parent));

        assert_eq!(left_child.and_then(|lc| lc.as_ref().left), left_sub_child);
        assert_eq!(left_child, left_sub_child.and_then(|c| c.as_ref().parent));
    }

    #[test]
    fn should_remove_node_with_two_childen_while_retaining_relationships() {
        let node_values = [10, 5, 1, 15];
        let tree = node_values
            .to_vec()
            .into_iter()
            .fold(BinaryTree::default(), |tree, x| tree.insert(x))
            .remove(&5);

        let root = unsafe { tree.find_nearest_node(&10).hit_then(|node| node) };

        let right_child = unsafe { tree.find_nearest_node(&15).hit_then(|node| node) };
        let left_child = unsafe { tree.find_nearest_node(&1).hit_then(|node| node) };

        assert_eq!(root, left_child.and_then(|c| c.as_ref().parent));
        assert_eq!(root, right_child.and_then(|c| c.as_ref().parent));
    }

    #[test]
    fn should_retain_order_after_deletion() {
        let tree = (0..1024)
            .rev()
            .fold(BinaryTree::default(), |tree, x| tree.insert(x))
            .remove(&511)
            .remove(&512);

        let received: Vec<u16> = tree.traverse_in_order().copied().collect();
        // skip 511 and 512
        let expected: Vec<u16> = (0..511).chain(513..1024).collect();
        assert_eq!(expected, received);
    }
}