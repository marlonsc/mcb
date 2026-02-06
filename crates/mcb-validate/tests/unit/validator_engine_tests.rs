//! Unit tests for ValidatorEngine.
//!
//! Moved from inline tests in src/engines/validator_engine.rs.

use mcb_validate::engines::ValidatorEngine;

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
        "engine": "rust-rule-engine",
        "config": {
            "crate_name": "test-crate",
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
        "engine": "rust-rule-engine"
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
