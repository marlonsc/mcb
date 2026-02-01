//! Unit tests for AST visitor
//!
//! Tests for `KindCollector` and `KindCounter` functionality.

use mcb_ast_utils::visitor::{KindCollector, KindCounter};

#[test]
fn test_kind_collector() {
    let collector = KindCollector::new("function_item");
    assert!(collector.matches.is_empty());
    assert_eq!(collector.target_kind, "function_item");
}

#[test]
fn test_kind_counter() {
    let counter = KindCounter::new();
    assert_eq!(counter.count("function"), 0);
}
