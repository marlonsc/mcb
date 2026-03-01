//! Tests for `EmbeddedRules` ensuring all bundled YAML files are valid.

use mcb_validate::embedded_rules::EmbeddedRules;
use rstest::rstest;

#[rstest]
#[test]
fn all_embedded_yaml_files_are_non_empty() {
    let rules = EmbeddedRules::all_yaml();
    assert!(!rules.is_empty(), "embedded rules list should not be empty");

    for (path, content) in &rules {
        assert!(
            !content.is_empty(),
            "embedded YAML file '{path}' must not be empty"
        );
    }
}

#[rstest]
#[test]
fn all_embedded_yaml_files_are_valid_yaml() {
    for (path, content) in &EmbeddedRules::all_yaml() {
        let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(content);
        assert!(
            parsed.is_ok(),
            "embedded YAML file '{path}' is not valid YAML: {}",
            parsed.unwrap_err()
        );
    }
}

#[rstest]
#[test]
fn rule_yaml_excludes_templates() {
    let rules = EmbeddedRules::rule_yaml();
    for (path, _) in &rules {
        assert!(
            !path.contains("/templates/"),
            "rule_yaml() should exclude templates, but found: {path}"
        );
    }
}

#[rstest]
#[test]
fn schema_json_is_non_empty_and_valid() {
    let schema = EmbeddedRules::SCHEMA_JSON;
    assert!(!schema.is_empty(), "schema JSON must not be empty");

    let parsed: Result<serde_json::Value, _> = serde_json::from_str(schema);
    assert!(
        parsed.is_ok(),
        "schema JSON is not valid: {}",
        parsed.unwrap_err()
    );
}
