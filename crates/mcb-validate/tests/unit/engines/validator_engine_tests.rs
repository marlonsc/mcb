//! Unit tests for `ValidatorEngine`.

use mcb_validate::engines::ValidatorEngine;
use rstest::rstest;

#[rstest]
#[case("architecture", "rust-rule-engine", true)]
#[case("invalid_category", "rust-rule-engine", false)]
#[case("architecture", "invalid_engine", false)]
fn validate_rule_config(
    #[case] category: &str,
    #[case] engine_name: &str,
    #[case] expected_ok: bool,
) {
    let engine = ValidatorEngine::new();

    let rule = serde_json::json!({
        "id": "TEST001",
        "name": "Test Rule",
        "category": category,
        "severity": "error",
        "description": "This is a test rule with enough description",
        "rationale": "This rule exists for testing purposes and has enough rationale",
        "engine": engine_name,
        "config": {
            "crate_name": "test-crate",
            "forbidden_prefixes": ["test"]
        }
    });

    assert_eq!(engine.validate_rule_definition(&rule).is_ok(), expected_ok);
}
