//! Tests for expression engine.
//!
//! Uses `create_rule_context_with_files` from shared helpers to eliminate
//! the duplicated `create_test_context` function, and `SNIPPET_*` constants
//! for inline file contents.

use std::collections::HashMap;

use mcb_validate::engines::expression_engine::ExpressionEngine;
use rstest::*;

use crate::test_constants::{SNIPPET_LIB_RS, SNIPPET_MAIN_RS};
use crate::test_utils::create_rule_context_with_files;

#[fixture]
fn expression_context() -> mcb_validate::engines::hybrid_engine::RuleContext {
    create_rule_context_with_files(&[
        ("src/main.rs", SNIPPET_MAIN_RS),
        ("src/lib.rs", SNIPPET_LIB_RS),
    ])
}

#[rstest]
#[case("file_count == 2", true)]
#[case("file_count > 10", false)]
#[case("has_unwrap == false", true)]
fn expression_evaluation(
    expression_context: mcb_validate::engines::hybrid_engine::RuleContext,
    #[case] expression: &str,
    #[case] expected: bool,
) {
    let engine = ExpressionEngine::new();
    let result = engine.evaluate_expression(expression, &expression_context);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_custom_variables() {
    let engine = ExpressionEngine::new();
    let mut variables = HashMap::new();
    variables.insert("x".to_string(), serde_json::json!(10));
    variables.insert("y".to_string(), serde_json::json!(5));

    let result = engine.evaluate_with_variables("x > y", &variables);
    assert!(result.is_ok());
    assert!(result.unwrap());

    let result = engine.evaluate_with_variables("x + y == 15", &variables);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_invalid_expression() {
    let engine = ExpressionEngine::new();
    let context = create_rule_context_with_files(&[
        ("src/main.rs", SNIPPET_MAIN_RS),
        ("src/lib.rs", SNIPPET_LIB_RS),
    ]);

    let result = engine.evaluate_expression("undefined_var > 0", &context);
    assert!(result.is_err());
}
