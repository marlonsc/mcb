//! Unit tests for cursor utilities
//!
//! Tests for `CursorUtils` functionality.

use mcb_ast_utils::cursor::CursorUtils;
use mcb_ast_utils::walker::TreeWalker;

use super::common::parse_rust;

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

    let let_nodes: Vec<_> = TreeWalker::find_by_kind(root, "let_declaration");
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
