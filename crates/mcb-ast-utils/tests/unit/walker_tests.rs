//! Unit tests for tree walker
//!
//! Tests for `TreeWalker` functionality.

use mcb_ast_utils::visitor::KindCounter;
use mcb_ast_utils::walker::TreeWalker;
use rstest::*;

fn parse_rust_code(code: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Error loading Rust grammar");
    parser.parse(code, None).unwrap()
}

#[test]
fn depth_root_is_zero() {
    let tree = parse_rust_code("fn main() { let x = 1; }");
    assert_eq!(TreeWalker::depth(tree.root_node()), 0);
}

#[test]
fn depth_nested_node_is_positive() {
    let tree = parse_rust_code("fn main() { let x = 1; }");
    let nodes = TreeWalker::find_by_kind(tree.root_node(), "let_declaration");
    assert!(!nodes.is_empty());
    assert!(TreeWalker::depth(nodes[0]) > 0);
}

#[rstest]
#[case("fn foo() {} fn bar() {}", "function_item", 2)]
#[case("fn main() { let x = 1; let y = 2; }", "let_declaration", 2)]
#[case("struct A {} struct B {} struct C {}", "struct_item", 3)]
fn find_by_kind_counts(#[case] code: &str, #[case] kind: &str, #[case] expected: usize) {
    let tree = parse_rust_code(code);
    let found = TreeWalker::find_by_kind(tree.root_node(), kind);
    assert_eq!(found.len(), expected);
}

#[test]
fn find_first_returns_some() {
    let tree = parse_rust_code("fn main() { let x = 1; let y = 2; }");
    let first_let = TreeWalker::find_first(tree.root_node(), "let_declaration");
    assert!(first_let.is_some());
}

#[test]
fn walk_with_counter() {
    let code = "fn main() { let x = 1; }";
    let tree = parse_rust_code(code);

    let mut counter = KindCounter::new();
    let mut ctx = ();
    TreeWalker::walk(&tree, code.as_bytes(), &mut counter, &mut ctx);

    assert!(counter.count("function_item") >= 1);
    assert!(counter.count("let_declaration") >= 1);
}

#[test]
fn is_inside_kind() {
    let tree = parse_rust_code("fn main() { let x = 1; }");
    let let_nodes = TreeWalker::find_by_kind(tree.root_node(), "let_declaration");
    assert!(!let_nodes.is_empty());
    assert!(TreeWalker::is_inside_kind(let_nodes[0], "function_item"));
}
