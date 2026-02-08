//! Cursor Utilities
//!
//! Provides utilities for working with tree-sitter cursors.

use tree_sitter::{Node, TreeCursor};

/// Cursor utilities for tree-sitter navigation
pub struct CursorUtils;

impl CursorUtils {
    /// Get the path from root to the current node
    ///
    /// Returns a vector of (`node_kind`, `child_index`) pairs.
    pub fn path_to_root(node: Node<'_>) -> Vec<(String, usize)> {
        let mut path = Vec::new();
        let mut current = node;

        while let Some(parent) = current.parent() {
            // Find the index of current among parent's children
            let mut cursor = parent.walk();
            let mut idx = 0;
            if cursor.goto_first_child() {
                loop {
                    if cursor.node().id() == current.id() {
                        path.push((current.kind().to_string(), idx));
                        break;
                    }
                    idx += 1;
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            current = parent;
        }

        path.push((current.kind().to_string(), 0));
        path.reverse();
        path
    }

    /// Get all siblings of a node
    pub fn siblings(node: Node<'_>) -> Vec<Node<'_>> {
        let Some(parent) = node.parent() else {
            return Vec::new();
        };

        let mut siblings = Vec::new();
        let mut cursor = parent.walk();

        if cursor.goto_first_child() {
            loop {
                let sibling = cursor.node();
                if sibling.id() != node.id() {
                    siblings.push(sibling);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        siblings
    }

    /// Get the previous sibling of a node
    pub fn prev_sibling(node: Node<'_>) -> Option<Node<'_>> {
        node.prev_sibling()
    }

    /// Get the next sibling of a node
    pub fn next_sibling(node: Node<'_>) -> Option<Node<'_>> {
        node.next_sibling()
    }

    /// Count children of a specific kind
    pub fn count_children_of_kind(node: Node<'_>, kind: &str) -> usize {
        let mut count = 0;
        Self::for_each_child(&mut node.walk(), |child| {
            if child.kind() == kind {
                count += 1;
            }
        });
        count
    }

    /// Get all children of a specific kind
    pub fn children_of_kind<'a>(node: Node<'a>, kind: &str) -> Vec<Node<'a>> {
        let mut children = Vec::new();
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == kind {
                    children.push(cursor.node());
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        children
    }

    /// Get the first child of a specific kind
    pub fn first_child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == kind {
                    return Some(cursor.node());
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        None
    }

    /// Get named children (excluding anonymous nodes like punctuation)
    pub fn named_children(node: Node<'_>) -> Vec<Node<'_>> {
        (0..node.named_child_count())
            .filter_map(|i| u32::try_from(i).ok().and_then(|u| node.named_child(u)))
            .collect()
    }

    /// Get child by field name
    pub fn child_by_field<'a>(node: Node<'a>, field: &str) -> Option<Node<'a>> {
        node.child_by_field_name(field)
    }

    /// Iterate over a cursor's children
    pub fn for_each_child<F>(cursor: &mut TreeCursor<'_>, mut f: F)
    where
        F: FnMut(Node<'_>),
    {
        if cursor.goto_first_child() {
            loop {
                f(cursor.node());
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }
}
