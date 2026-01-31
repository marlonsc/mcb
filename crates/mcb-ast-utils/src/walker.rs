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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visitor::KindCounter;

    #[test]
    fn test_depth() {
        // Create a simple tree for testing
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");

        let code = "fn main() { let x = 1; }";
        let tree = parser.parse(code, None).unwrap();

        let root = tree.root_node();
        assert_eq!(TreeWalker::depth(root), 0);

        // Find a deeper node
        let nodes = TreeWalker::find_by_kind(root, "let_declaration");
        assert!(!nodes.is_empty());
        assert!(TreeWalker::depth(nodes[0]) > 0);
    }

    #[test]
    fn test_find_by_kind() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");

        let code = "fn foo() {} fn bar() {}";
        let tree = parser.parse(code, None).unwrap();

        let functions = TreeWalker::find_by_kind(tree.root_node(), "function_item");
        assert_eq!(functions.len(), 2);
    }

    #[test]
    fn test_find_first() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");

        let code = "fn main() { let x = 1; let y = 2; }";
        let tree = parser.parse(code, None).unwrap();

        let first_let = TreeWalker::find_first(tree.root_node(), "let_declaration");
        assert!(first_let.is_some());
    }

    #[test]
    fn test_walk_with_counter() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");

        let code = "fn main() { let x = 1; }";
        let tree = parser.parse(code, None).unwrap();

        let mut counter = KindCounter::new();
        let mut ctx = ();
        TreeWalker::walk(&tree, code.as_bytes(), &mut counter, &mut ctx);

        assert!(counter.count("function_item") >= 1);
        assert!(counter.count("let_declaration") >= 1);
    }

    #[test]
    fn test_is_inside_kind() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");

        let code = "fn main() { let x = 1; }";
        let tree = parser.parse(code, None).unwrap();

        let let_nodes = TreeWalker::find_by_kind(tree.root_node(), "let_declaration");
        assert!(!let_nodes.is_empty());

        // let_declaration should be inside function_item
        assert!(TreeWalker::is_inside_kind(let_nodes[0], "function_item"));
    }
}
