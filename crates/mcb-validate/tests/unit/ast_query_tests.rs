//! Unit tests for AST query (AstQueryBuilder, AstQueryPatterns).
//!
//! Moved from inline tests in src/ast/query.rs.

use mcb_validate::ast::query::{AstQueryBuilder, AstQueryPatterns, QueryCondition};

#[test]
fn test_query_builder() {
    let query = AstQueryBuilder::new("rust", "function_item")
        .with_condition(QueryCondition::Custom {
            name: "has_no_docstring".to_string(),
        })
        .message("Function needs documentation")
        .severity("warning")
        .build();

    assert_eq!(query.language, "rust");
    assert_eq!(query.node_type, "function_item");
    assert_eq!(query.message, "Function needs documentation");
    assert_eq!(query.severity, "warning");
    assert_eq!(query.conditions.len(), 1);
}

#[test]
fn test_query_patterns() {
    let query = AstQueryPatterns::undocumented_functions("rust");
    assert_eq!(query.language, "rust");
    assert_eq!(query.node_type, "function_item");
    assert_eq!(query.message, "Functions must be documented");
}

#[test]
fn test_unwrap_pattern() {
    let query = AstQueryPatterns::unwrap_usage("rust");
    assert_eq!(query.language, "rust");
    assert_eq!(query.node_type, "call_expression");
    assert_eq!(query.message, "Avoid unwrap() in production code");
    assert_eq!(query.severity, "error");
}
