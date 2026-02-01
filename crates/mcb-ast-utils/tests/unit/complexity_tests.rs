//! Unit tests for complexity analysis
//!
//! Tests for `ComplexityAnalyzer` functionality.

use mcb_ast_utils::complexity::{ComplexityAnalyzer, count_parameters};
use mcb_ast_utils::walker::TreeWalker;
use mcb_language_support::language::LanguageId;
use tree_sitter::Parser;

fn parse_rust(code: &str) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Error loading Rust grammar");
    parser.parse(code, None).unwrap()
}

#[test]
fn test_simple_function_complexity() {
    let code = "fn simple() { let x = 1; }";
    let tree = parse_rust(code);
    let metrics = ComplexityAnalyzer::analyze(tree.root_node(), LanguageId::Rust);

    assert_eq!(metrics.cyclomatic, 1); // Base complexity only
    assert_eq!(metrics.loops, 0);
    assert_eq!(metrics.branches, 0);
}

#[test]
fn test_if_complexity() {
    let code = "fn test() { if true { 1 } else { 2 } }";
    let tree = parse_rust(code);
    let metrics = ComplexityAnalyzer::analyze(tree.root_node(), LanguageId::Rust);

    // if adds 1, else adds 1
    assert!(metrics.cyclomatic >= 2);
    assert!(metrics.branches >= 1);
}

#[test]
fn test_loop_complexity() {
    let code = "fn test() { for i in 0..10 { } while true { } }";
    let tree = parse_rust(code);
    let metrics = ComplexityAnalyzer::analyze(tree.root_node(), LanguageId::Rust);

    assert!(metrics.cyclomatic >= 3); // 1 base + 2 loops
    assert_eq!(metrics.loops, 2);
}

#[test]
fn test_nesting_depth() {
    let code = "fn test() { if true { if true { 1 } } }";
    let tree = parse_rust(code);
    let metrics = ComplexityAnalyzer::analyze(tree.root_node(), LanguageId::Rust);

    assert!(metrics.max_nesting >= 2);
}

#[test]
fn test_count_parameters() {
    let code = "fn test(a: i32, b: i32, c: i32) {}";
    let tree = parse_rust(code);

    let functions = TreeWalker::find_by_kind(tree.root_node(), "function_item");
    assert_eq!(functions.len(), 1);

    let count = count_parameters(functions[0]);
    assert_eq!(count, 3);
}
