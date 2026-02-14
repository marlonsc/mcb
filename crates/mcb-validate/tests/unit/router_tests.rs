//! Unit tests for `RuleEngineRouter`.
//!
//! Uses `ENGINE_NAME_*` constants instead of hardcoded engine strings,
//! and `DOMAIN_CRATE` / `FORBIDDEN_PREFIX_PATTERN` from shared constants.

use mcb_validate::engines::{RoutedEngine, RuleEngineRouter};
use rstest::rstest;
use serde_json::json;

use crate::test_constants::*;

#[rstest]
#[case(
    json!({"engine": ENGINE_NAME_RUST_RULE, "rule": "rule Test { when true then Action(); }"}),
    RoutedEngine::Rete
)]
#[case(
    json!({"rule": format!(r#"rule DomainCheck \"Check domain\" {{ when Crate(name == \"{DOMAIN_CRATE}\") then Violation(\"Error\"); }}"#)}),
    RoutedEngine::Rete
)]
#[case(
    json!({"expression": "file_count > 100", "message": "Too many files"}),
    RoutedEngine::Expression
)]
#[case(
    json!({"condition": {"all": [{"fact_type": "file", "field": "path", "operator": "matches", "value": "*.rs"}]}, "action": {"violation": {"message": "Rule triggered"}}}),
    RoutedEngine::RustyRules
)]
#[case(
    json!({"type": "cargo_dependencies", "pattern": FORBIDDEN_PREFIX_PATTERN}),
    RoutedEngine::RustyRules
)]
#[test]
fn test_detect_engine(#[case] rule: serde_json::Value, #[case] expected: RoutedEngine) {
    let router = RuleEngineRouter::new();
    assert_eq!(router.detect_engine(&rule), expected);
}

#[test]
fn test_validate_rete_rule() {
    let router = RuleEngineRouter::new();

    let valid_rule = json!({
        "engine": ENGINE_NAME_RETE,
        "rule": "rule Test { when true then Action(); }"
    });
    assert!(router.validate_rule(&valid_rule).is_ok());

    let invalid_rule = json!({
        "engine": ENGINE_NAME_RETE,
        "message": "Something"
    });
    assert!(router.validate_rule(&invalid_rule).is_err());
}

#[test]
fn test_validate_expression_rule() {
    let router = RuleEngineRouter::new();

    let valid_rule = json!({
        "engine": ENGINE_NAME_EXPRESSION,
        "expression": "x > 5"
    });
    assert!(router.validate_rule(&valid_rule).is_ok());

    let invalid_rule = json!({
        "engine": ENGINE_NAME_EXPRESSION,
        "message": "Something"
    });
    assert!(router.validate_rule(&invalid_rule).is_err());
}
