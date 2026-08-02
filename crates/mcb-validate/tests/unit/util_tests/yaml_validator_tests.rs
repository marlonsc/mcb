//! Unit tests for YAML rule validator.

use mcb_validate::rules::YamlRuleValidator;
use rstest::rstest;
use rstest::*;
use serde_json::json;

#[fixture]
fn validator() -> Result<YamlRuleValidator, Box<dyn std::error::Error>> {
    Ok(YamlRuleValidator::new()?)
}

fn base_rule() -> serde_json::Value {
    json!({
        "schema": "rule/v1",
        "id": "TEST001",
        "name": "Test Rule",
        "category": "architecture",
        "severity": "error",
        "description": "Valid description",
        "rationale": "Valid rationale",
        "engine": "rust-rule-engine",
        "config": {
            "crate_name": "test-crate"
        },
        "rule": {
            "type": "cargo_dependencies"
        }
    })
}

#[rstest]
fn test_schema_loading() {
    let validator = YamlRuleValidator::new();
    assert!(validator.is_ok());
}

#[rstest]
fn test_valid_rule_validation(
    validator: Result<YamlRuleValidator, Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let validator = validator?;
    let valid_rule = base_rule();
    assert!(validator.validate_rule(&valid_rule).is_ok());
    Ok(())
}

#[rstest]
#[case("category", "invalid_category")]
#[case("severity", "invalid_severity")]
#[case("engine", "invalid_engine")]
fn test_invalid_field_values(
    validator: Result<YamlRuleValidator, Box<dyn std::error::Error>>,
    #[case] field: &str,
    #[case] value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let validator = validator?;
    let mut invalid_rule = base_rule();
    invalid_rule[field] = value.into();
    assert!(
        validator.validate_rule(&invalid_rule).is_err(),
        "Expected error for invalid {field}"
    );
    Ok(())
}

#[rstest]
fn test_missing_required_fields(
    validator: Result<YamlRuleValidator, Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let validator = validator?;
    let invalid_rule = json!({
        "name": "Test Rule",
        "category": "architecture"
    });
    assert!(validator.validate_rule(&invalid_rule).is_err());
    Ok(())
}
