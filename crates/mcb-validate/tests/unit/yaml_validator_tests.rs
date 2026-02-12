//! Unit tests for YAML rule validator.

use mcb_validate::rules::YamlRuleValidator;

#[test]
fn test_schema_loading() {
    let validator = YamlRuleValidator::new();
    assert!(validator.is_ok());
}

#[test]
fn test_valid_rule_validation() {
    let validator = YamlRuleValidator::new().unwrap();

    let valid_rule = serde_json::json!({
        "schema": "rule/v1",
        "id": "TEST001",
        "name": "Test Rule",
        "category": "architecture",
        "severity": "error",
        "description": "This is a test rule with enough description to pass validation requirements",
        "rationale": "This rule exists for testing purposes and has enough rationale text",
        "engine": "rust-rule-engine",
        "config": {
            "crate_name": "test-crate"
        },
        "rule": {
            "type": "cargo_dependencies"
        }
    });

    assert!(validator.validate_rule(&valid_rule).is_ok());
}

#[test]
fn test_invalid_rule_validation() {
    let validator = YamlRuleValidator::new().unwrap();

    let invalid_rule = serde_json::json!({
        "name": "Test Rule",
        "category": "architecture"
    });

    assert!(validator.validate_rule(&invalid_rule).is_err());
}

#[test]
fn test_invalid_category() {
    let validator = YamlRuleValidator::new().unwrap();

    let invalid_rule = serde_json::json!({
        "schema": "rule/v1",
        "id": "TEST001",
        "name": "Test Rule",
        "category": "invalid_category",
        "severity": "error",
        "description": "This is a test rule description",
        "rationale": "This is the rationale for the rule",
        "engine": "rust-rule-engine",
        "rule": {}
    });

    assert!(validator.validate_rule(&invalid_rule).is_err());
}

#[test]
fn test_invalid_severity() {
    let validator = YamlRuleValidator::new().unwrap();

    let invalid_rule = serde_json::json!({
        "schema": "rule/v1",
        "id": "TEST001",
        "name": "Test Rule",
        "category": "architecture",
        "severity": "invalid_severity",
        "description": "This is a test rule description",
        "rationale": "This is the rationale for the rule",
        "engine": "rust-rule-engine",
        "rule": {}
    });

    assert!(validator.validate_rule(&invalid_rule).is_err());
}

#[test]
fn test_invalid_engine() {
    let validator = YamlRuleValidator::new().unwrap();

    let invalid_rule = serde_json::json!({
        "schema": "rule/v1",
        "id": "TEST001",
        "name": "Test Rule",
        "category": "architecture",
        "severity": "error",
        "description": "This is a test rule description",
        "rationale": "This is the rationale for the rule",
        "engine": "invalid_engine",
        "rule": {}
    });

    assert!(validator.validate_rule(&invalid_rule).is_err());
}
