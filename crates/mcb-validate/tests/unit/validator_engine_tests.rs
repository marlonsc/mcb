//! Unit tests for `ValidatorEngine`.

use mcb_validate::engines::ValidatorEngine;

use crate::test_constants::*;

#[test]
fn test_valid_rule_config() {
    let engine = ValidatorEngine::new();

    let valid_rule = serde_json::json!({
        "id": "TEST001",
        "name": "Test Rule",
        "category": "architecture",
        "severity": "error",
        "description": "This is a test rule with enough description",
        "rationale": "This rule exists for testing purposes and has enough rationale",
        "engine": ENGINE_NAME_RUST_RULE,
        "config": {
            "crate_name": TEST_SUBJECT_CRATE,
            "forbidden_prefixes": ["test"]
        }
    });

    assert!(engine.validate_rule_definition(&valid_rule).is_ok());
}

#[test]
fn test_invalid_category() {
    let engine = ValidatorEngine::new();

    let invalid_rule = serde_json::json!({
        "id": "TEST001",
        "name": "Test Rule",
        "category": "invalid_category",
        "severity": "error",
        "description": "This is a test rule",
        "rationale": "This rule exists for testing",
        "engine": ENGINE_NAME_RUST_RULE
    });

    assert!(engine.validate_rule_definition(&invalid_rule).is_err());
}

#[test]
fn test_invalid_engine() {
    let engine = ValidatorEngine::new();

    let invalid_rule = serde_json::json!({
        "id": "TEST001",
        "name": "Test Rule",
        "category": "architecture",
        "severity": "error",
        "description": "This is a test rule",
        "rationale": "This rule exists for testing",
        "engine": "invalid_engine"
    });

    assert!(engine.validate_rule_definition(&invalid_rule).is_err());
}
