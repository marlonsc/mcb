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
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == kind {
                    count += 1;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

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
    #[allow(
        clippy::cast_possible_truncation,
        reason = "Child count is always small"
    )]
    pub fn named_children(node: Node<'_>) -> Vec<Node<'_>> {
        (0..node.named_child_count())
            .filter_map(|i| node.named_child(i as u32))
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

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_rust(code: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn test_siblings() {
        let tree = parse_rust("fn foo() {} fn bar() {}");
        let root = tree.root_node();

        let functions: Vec<_> = CursorUtils::children_of_kind(root, "function_item");
        assert_eq!(functions.len(), 2);

        let siblings = CursorUtils::siblings(functions[0]);
        assert_eq!(siblings.len(), 1);
        assert_eq!(siblings[0].kind(), "function_item");
    }

    #[test]
    fn test_count_children_of_kind() {
        let tree = parse_rust("fn main() { let x = 1; let y = 2; }");
        let root = tree.root_node();

        let functions: Vec<_> = CursorUtils::children_of_kind(root, "function_item");
        assert_eq!(functions.len(), 1);

        // Get the block inside the function
        let block = CursorUtils::first_child_of_kind(functions[0], "block");
        assert!(block.is_some());

        let let_count = CursorUtils::count_children_of_kind(block.unwrap(), "let_declaration");
        assert_eq!(let_count, 2);
    }

    #[test]
    fn test_path_to_root() {
        let tree = parse_rust("fn main() { let x = 1; }");
        let root = tree.root_node();

        let let_nodes: Vec<_> = crate::walker::TreeWalker::find_by_kind(root, "let_declaration");
        assert!(!let_nodes.is_empty());

        let path = CursorUtils::path_to_root(let_nodes[0]);
        assert!(!path.is_empty());
        assert_eq!(path[0].0, "source_file");
    }

    #[test]
    fn test_named_children() {
        let tree = parse_rust("fn foo(x: i32, y: i32) {}");
        let root = tree.root_node();

        let functions: Vec<_> = CursorUtils::children_of_kind(root, "function_item");
        let named = CursorUtils::named_children(functions[0]);

        // Should have name, parameters, and block as named children
        assert!(named.len() >= 2);
    }

    #[test]
    fn test_child_by_field() {
        let tree = parse_rust("fn foo() {}");
        let root = tree.root_node();

        let functions: Vec<_> = CursorUtils::children_of_kind(root, "function_item");
        let name = CursorUtils::child_by_field(functions[0], "name");

        assert!(name.is_some());
        assert_eq!(name.unwrap().kind(), "identifier");
    }
}
