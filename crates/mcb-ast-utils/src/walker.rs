//! Tree Walker
//!
//! Provides utilities for walking AST trees using tree-sitter.

use tree_sitter::{Node, Tree};

use crate::visitor::NodeVisitor;

/// Tree walker for AST traversal
pub struct TreeWalker;

impl TreeWalker {
    /// Walk a tree with a visitor
    ///
    /// Performs a depth-first traversal of the tree, calling the visitor
    /// for each node.
    pub fn walk<V: NodeVisitor>(tree: &Tree, source: &[u8], visitor: &mut V, ctx: &mut V::Context) {
        Self::walk_node(tree.root_node(), source, visitor, ctx);
    }

    /// Walk starting from a specific node
    pub fn walk_node<V: NodeVisitor>(
        node: Node<'_>,
        source: &[u8],
        visitor: &mut V,
        ctx: &mut V::Context,
    ) {
        let should_continue = visitor.visit(node, source, ctx);

        if should_continue {
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    Self::walk_node(cursor.node(), source, visitor, ctx);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        visitor.leave(node, source, ctx);
    }

    /// Find all nodes of a specific kind
    pub fn find_by_kind<'a>(node: Node<'a>, kind: &str) -> Vec<Node<'a>> {
        let mut results = Vec::new();
        Self::find_by_kind_recursive(node, kind, &mut results);
        results
    }

    fn find_by_kind_recursive<'a>(node: Node<'a>, kind: &str, results: &mut Vec<Node<'a>>) {
        if node.kind() == kind {
            results.push(node);
        }

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                Self::find_by_kind_recursive(cursor.node(), kind, results);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    /// Find the first node of a specific kind
    pub fn find_first<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
        if node.kind() == kind {
            return Some(node);
        }

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if let Some(found) = Self::find_first(cursor.node(), kind) {
                    return Some(found);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        None
    }

    /// Get the depth of a node in the tree
    pub fn depth(node: Node<'_>) -> usize {
        let mut depth = 0;
        let mut current = node;
        while let Some(parent) = current.parent() {
            depth += 1;
            current = parent;
        }
        depth
    }

    /// Get all ancestors of a node
    pub fn ancestors(node: Node<'_>) -> Vec<Node<'_>> {
        let mut ancestors = Vec::new();
        let mut current = node;
        while let Some(parent) = current.parent() {
            ancestors.push(parent);
            current = parent;
        }
        ancestors
    }

    /// Check if node is inside another node of a specific kind
    pub fn is_inside_kind(node: Node<'_>, kind: &str) -> bool {
        Self::ancestors(node).iter().any(|n| n.kind() == kind)
    }
}

// Tests moved to tests/unit/walker_tests.rs
