//! Unit tests for AST query (`AstQueryBuilder`, `AstQueryPatterns`).

use mcb_validate::ast::query::{AstQueryBuilder, AstQueryPatterns, QueryCondition};
use rstest::rstest;

#[rstest]
#[test]
fn test_query_builder() {
    let query = AstQueryBuilder::new("rust", "function_item")
        .with_condition(QueryCondition::Custom {
            name: "has_no_docstring".to_owned(),
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

#[rstest]
#[case(
    "undocumented",
    "function_item",
    "Functions must be documented",
    "warning"
)]
#[case(
    "unwrap",
    "call_expression",
    "Avoid unwrap() in production code",
    "error"
)]
fn query_patterns(
    #[case] pattern: &str,
    #[case] expected_node_type: &str,
    #[case] expected_message: &str,
    #[case] expected_severity: &str,
) {
    let query = match pattern {
        "undocumented" => AstQueryPatterns::undocumented_functions("rust"),
        _ => AstQueryPatterns::unwrap_usage("rust"),
    };

    assert_eq!(query.language, "rust");
    assert_eq!(query.node_type, expected_node_type);
    assert_eq!(query.message, expected_message);
    assert_eq!(query.severity, expected_severity);
}
