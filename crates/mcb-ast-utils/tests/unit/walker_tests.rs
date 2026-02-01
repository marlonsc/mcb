//! Unit tests for tree walker
//!
//! Tests for `TreeWalker` functionality.

use mcb_ast_utils::visitor::KindCounter;
use mcb_ast_utils::walker::TreeWalker;

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
