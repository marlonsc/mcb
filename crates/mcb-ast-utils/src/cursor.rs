//! Cursor Utilities
//!
//! Provides utilities for working with tree-sitter cursors.

use tree_sitter::Node;

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
            let mut cursor = parent.walk();
            let idx = parent
                .children(&mut cursor)
                .position(|child| child.id() == current.id())
                .unwrap_or(0);

            path.push((current.kind().to_string(), idx));
            current = parent;
        }

        path.push((current.kind().to_string(), 0));
        path.reverse();
        path
    }

    /// Get all siblings of a node
    pub fn siblings(node: Node<'_>) -> Vec<Node<'_>> {
        node.parent()
            .map(|parent| {
                let mut cursor = parent.walk();
                parent
                    .children(&mut cursor)
                    .filter(|child| child.id() != node.id())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the next sibling of a node
    pub fn next_sibling(node: Node<'_>) -> Option<Node<'_>> {
        node.next_sibling()
    }

    /// Count children of a specific kind
    pub fn count_children_of_kind(node: Node<'_>, kind: &str) -> usize {
        let mut cursor = node.walk();
        node.children(&mut cursor)
            .filter(|child| child.kind() == kind)
            .count()
    }

    /// Get all children of a specific kind
    pub fn children_of_kind<'a>(node: Node<'a>, kind: &str) -> Vec<Node<'a>> {
        let mut cursor = node.walk();
        node.children(&mut cursor)
            .filter(|child| child.kind() == kind)
            .collect()
    }

    /// Get the first child of a specific kind
    pub fn first_child_of_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
        let mut cursor = node.walk();
        node.children(&mut cursor)
            .find(|child| child.kind() == kind)
    }

    /// Get named children (excluding anonymous nodes like punctuation)
    pub fn named_children(node: Node<'_>) -> Vec<Node<'_>> {
        let mut cursor = node.walk();
        node.named_children(&mut cursor).collect()
    }

    /// Get child by field name
    pub fn child_by_field<'a>(node: Node<'a>, field: &str) -> Option<Node<'a>> {
        node.child_by_field_name(field)
    }
}
