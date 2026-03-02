//! Unit tests for `RuleEngineRouter`.
//!
//! Uses `ENGINE_NAME_*` constants instead of hardcoded engine strings,
//! and `DOMAIN_CRATE` / `FORBIDDEN_PREFIX_PATTERN` from shared constants.

use mcb_validate::engines::{RoutedEngine, RuleEngineRouter};
use rstest::rstest;
use serde_json::json;

use mcb_domain::utils::test_constants::*;

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
fn detect_engine(#[case] rule: serde_json::Value, #[case] expected: RoutedEngine) {
    assert_eq!(RuleEngineRouter::detect_engine(&rule), expected);
}

#[rstest]
#[case(
    json!({"engine": ENGINE_NAME_RETE, "rule": "rule Test { when true then Action(); }"}),
    true
)]
#[case(json!({"engine": ENGINE_NAME_RETE, "message": "Something"}), false)]
#[case(
    json!({"engine": ENGINE_NAME_EXPRESSION, "expression": "x > 5"}),
    true
)]
#[case(
    json!({"engine": ENGINE_NAME_EXPRESSION, "message": "Something"}),
    false
)]
fn validate_rule(#[case] rule: serde_json::Value, #[case] expected: bool) {
    assert_eq!(RuleEngineRouter::validate_rule(&rule).is_ok(), expected);
}
