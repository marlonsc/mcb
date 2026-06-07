//! Tests for expression engine.
//!
//! Uses `create_rule_context_with_files` from shared helpers to eliminate
//! the duplicated `create_test_context` function, and `SNIPPET_*` constants
//! for inline file contents.

use rstest::rstest;
use std::collections::HashMap;

use mcb_validate::engines::expression_engine::ExpressionEngine;
use rstest::*;

use crate::utils::create_rule_context_with_files;
use crate::utils::test_constants::{SNIPPET_LIB_RS, SNIPPET_MAIN_RS};

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
    let value = result.expect("expression should evaluate");
    assert_eq!(value, expected);
}

#[rstest]
#[test]
fn test_custom_variables() {
    let engine = ExpressionEngine::new();
    let mut variables = HashMap::new();
    variables.insert("x".to_owned(), serde_json::json!(10));
    variables.insert("y".to_owned(), serde_json::json!(5));

    let result = engine.evaluate_with_variables("x > y", &variables);
    let value = result.expect("x > y should evaluate");
    assert!(value);

    let result = engine.evaluate_with_variables("x + y == 15", &variables);
    let value = result.expect("x + y == 15 should evaluate");
    assert!(value);
}

#[rstest]
#[test]
fn test_invalid_expression() {
    let engine = ExpressionEngine::new();
    let context = create_rule_context_with_files(&[
        ("src/main.rs", SNIPPET_MAIN_RS),
        ("src/lib.rs", SNIPPET_LIB_RS),
    ]);

    let result = engine.evaluate_expression("undefined_var > 0", &context);
    let err = result.expect_err("invalid expression should fail");
    assert!(
        err.to_string().contains("Expression evaluation error"),
        "unexpected error: {err}"
    );
}
