//! The actual splay tree implementation.
//!
//! This implementation has no generics, works only with trait objects, and
//! therefore does no monomorphization.  While the `pub struct SplayTree<T, O>`
//! users' API does use generics for ergonomics, it immediately erases types by
//! converting them to trait objects before calling into this `internal`
//! implementation. By erasing generic types, we keep code size
//! small. Therefore, it doesn't make sense to allow any of the `internal`
//! methods working with trait objects to be inlined, or else all our work would
//! be undone.

use super::Node;
use core::cmp;

/// Internal trait for anything that can be compared to a `Node`.
pub trait CompareToNode<'a> {
    /// Compare `self` to the value containing the given `Node`.
    ///
    /// # Safety
    ///
    /// Unsafe because implementers rely on only being called with nodes
    /// contained within the `NodeOffset::Value` container type they are
    /// expecting, and if given a random `Node`, then calling this will lead
    /// to unsafety.
    unsafe fn compare_to_node(&self, node: &'a Node<'a>) -> cmp::Ordering;
}

#[derive(Debug)]
pub struct SplayTree<'a> {
    root: Option<&'a Node<'a>>,
}

impl<'a> Default for SplayTree<'a> {
    #[inline]
    fn default() -> SplayTree<'a> {
        SplayTree { root: None }
    }
}

impl<'a> SplayTree<'a> {
    #[inline]
    pub const fn new() -> Self {
        SplayTree { root: None }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    #[inline]
    pub fn root(&self) -> Option<&'a Node<'a>> {
        self.root
    }

    #[inline(never)]
    pub unsafe fn find(&mut self, key: &dyn CompareToNode<'a>) -> Option<&'a Node<'a>> {
        match self.root {
            Some(root) => {
                let root = self.splay(root, key);
                if let cmp::Ordering::Equal = key.compare_to_node(root) {
                    Some(root)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    #[inline(never)]
    pub unsafe fn insert(&mut self, key: &dyn CompareToNode<'a>, node: &'a Node<'a>) -> bool {
        debug_assert!(node.left.get().is_none() && node.right.get().is_none());

        match self.root {
            Some(root) => {
                let root = self.splay(root, key);

                match key.compare_to_node(root) {
                    cmp::Ordering::Equal => return false,
                    cmp::Ordering::Less => {
                        node.left.set(root.left.get());
                        node.right.set(Some(root));
                        root.left.set(None);
                    }
                    cmp::Ordering::Greater => {
                        node.right.set(root.right.get());
                        node.left.set(Some(root));
                        root.right.set(None);
                    }
                }

                self.root = Some(node);
                true
            }
            None => {
                self.root = Some(node);
                true
            }
        }
    }

    #[inline(never)]
    pub unsafe fn remove(&mut self, key: &dyn CompareToNode<'a>) -> Option<&'a Node<'a>> {
        match self.root {
            Some(root) => {
                // Do a splay to move the node to the root, if it exists.
                let node = self.splay(root, key);
                self.root = None;
                if let cmp::Ordering::Equal = key.compare_to_node(node) {
                    // Ok, we found the node we want to remove. Disconnect it from
                    // the tree and fix up the new `self.root`.
                    match node.left.get() {
                        Some(node_left) => {
                            let right = node.right.get();
                            self.splay(node_left, key).right.set(right);
                        }
                        None => {
                            self.root = node.right.get();
                        }
                    }

                    node.left.set(None);
                    node.right.set(None);
                    return Some(node);
                }

                // The node we were trying to remove isn't in the tree.
                None
            }
            None => None,
        }
    }

    pub fn walk(&self, f: &mut dyn FnMut(&'a Node<'a>) -> bool) {
        if let Some(root) = self.root {
            root.walk(f);
        }
    }

    // The "simple top-down splay" routine from the paper.
    unsafe fn splay(
        &mut self,
        mut current: &'a Node<'a>,
        key: &dyn CompareToNode<'a>,
    ) -> &'a Node<'a> {
        let null = Node::default();
        let mut left = &null;
        let mut right = &null;

        loop {
            match key.compare_to_node(current) {
                cmp::Ordering::Less => {
                    match current.left.get() {
                        None => break,
                        Some(mut current_left) => {
                            if let cmp::Ordering::Less = key.compare_to_node(current_left) {
                                // Rotate right.
                                current.left.set(current_left.right.get());
                                current_left.right.set(Some(current));
                                current = current_left;
                                match current.left.get() {
                                    Some(l) => current_left = l,
                                    None => break,
                                }
                            }
                            // Link right.
                            right.left.set(Some(current));
                            right = current;
                            current = current_left;
                        }
                    }
                }
                cmp::Ordering::Greater => {
                    match current.right.get() {
                        None => break,
                        Some(mut current_right) => {
                            if let cmp::Ordering::Greater = key.compare_to_node(current_right) {
                                // Rotate left.
                                current.right.set(current_right.left.get());
                                current_right.left.set(Some(current));
                                current = current_right;
                                match current_right.right.get() {
                                    Some(r) => current_right = r,
                                    None => break,
                                }
                            }
                            // Link left.
                            left.right.set(Some(current));
                            left = current;
                            current = current_right;
                        }
                    }
                }
                cmp::Ordering::Equal => break,
            }
        }

        // Assemble.
        left.right.set(current.left.get());
        right.left.set(current.right.get());
        current.left.set(null.right.get());
        current.right.set(null.left.get());
        self.root = Some(current);
        current
    }
}
