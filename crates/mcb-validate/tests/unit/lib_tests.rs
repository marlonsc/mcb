//! Unit tests for `mcb_validate::lib` module

use std::path::PathBuf;

use mcb_validate::{Severity, ValidationConfig, ValidatorRegistry};
use rstest::*;

#[test]
fn test_severity_serialization() {
    let severity = Severity::Error;
    let json = serde_json::to_string(&severity).unwrap();
    assert_eq!(json, "\"Error\"");
}

#[test]
fn test_validation_config_creation() {
    let config = ValidationConfig::new("/workspace");
    assert_eq!(config.workspace_root.to_str().unwrap(), "/workspace");
    assert!(config.additional_src_paths.is_empty());
    assert!(config.exclude_patterns.is_empty());
}

#[test]
fn test_validation_config_builder() {
    let config = ValidationConfig::new("/workspace")
        .with_additional_path("../src")
        .with_additional_path("../legacy")
        .with_exclude_pattern("target/")
        .with_exclude_pattern("tests/fixtures/");

    assert_eq!(config.additional_src_paths.len(), 2);
    assert_eq!(config.exclude_patterns.len(), 2);
}

#[rstest]
#[case("/workspace/target/debug", true)]
#[case("/workspace/tests/fixtures/data.json", true)]
#[case("/workspace/src/lib.rs", false)]
fn validation_config_should_exclude(#[case] file: &str, #[case] expected: bool) {
    let config = ValidationConfig::new("/workspace")
        .with_exclude_pattern("target/")
        .with_exclude_pattern("fixtures/");

    assert_eq!(config.should_exclude(&PathBuf::from(file)), expected);
}

#[test]
fn test_validator_registry_with_config() {
    let config = ValidationConfig::new("/tmp/test-workspace")
        .with_additional_path("../legacy-src")
        .with_exclude_pattern("target/");

    let registry = ValidatorRegistry::standard_for(&config.workspace_root);

    assert_eq!(config.additional_src_paths.len(), 1);
    assert_eq!(config.exclude_patterns.len(), 1);
    assert!(!registry.validators().is_empty());
}
