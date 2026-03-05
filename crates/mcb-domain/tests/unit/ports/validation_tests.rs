//! Unit tests for validation port types and configuration.
//!
//! Tests cover: ValidationConfig construction, builder methods, path exclusion,
//! Severity serialization, ViolationCategory display, and NamedCheck/run_checks flow.

use mcb_domain::ports::validation::{
    NamedCheck, Severity, ValidationConfig, ViolationCategory, run_checks,
};
use rstest::rstest;

// ---------------------------------------------------------------------------
// ValidationConfig
// ---------------------------------------------------------------------------

#[rstest]
fn config_new_sets_workspace_root() {
    let config = ValidationConfig::new("/tmp/test_workspace");
    // canonicalize may adjust the path, but it should end with the expected dir
    let root_str = config.workspace_root.to_string_lossy();
    assert!(
        root_str.contains("test_workspace") || root_str.contains("tmp"),
        "workspace_root should contain the specified path: {root_str}"
    );
}

#[rstest]
fn config_defaults_empty_lists() {
    let config = ValidationConfig::new("/tmp");
    assert!(config.additional_src_paths.is_empty());
    assert!(config.exclude_patterns.is_empty());
}

#[rstest]
fn config_with_additional_path() {
    let config = ValidationConfig::new("/tmp").with_additional_path("/extra/src");
    assert_eq!(config.additional_src_paths.len(), 1);
    assert_eq!(
        config.additional_src_paths[0].to_string_lossy(),
        "/extra/src"
    );
}

#[rstest]
fn config_with_exclude_pattern() {
    let config = ValidationConfig::new("/tmp")
        .with_exclude_pattern("target/")
        .with_exclude_pattern("tests/fixtures/");
    assert_eq!(config.exclude_patterns.len(), 2);
    assert!(config.exclude_patterns.contains(&"target/".to_owned()));
}

#[rstest]
fn config_should_exclude_matching_path() {
    let config = ValidationConfig::new("/tmp")
        .with_exclude_pattern("target/")
        .with_exclude_pattern("node_modules/");

    assert!(config.should_exclude(std::path::Path::new("/project/target/debug/build")));
    assert!(config.should_exclude(std::path::Path::new("/project/node_modules/foo")));
    assert!(!config.should_exclude(std::path::Path::new("/project/src/main.rs")));
}

#[rstest]
fn config_should_exclude_empty_patterns() {
    let config = ValidationConfig::new("/tmp");
    assert!(!config.should_exclude(std::path::Path::new("/any/path")));
}

// ---------------------------------------------------------------------------
// Severity serialization
// ---------------------------------------------------------------------------

#[rstest]
#[case(Severity::Error, "\"ERROR\"")]
#[case(Severity::Warning, "\"WARNING\"")]
#[case(Severity::Info, "\"INFO\"")]
fn severity_serializes_uppercase(#[case] severity: Severity, #[case] expected: &str) {
    let json = serde_json::to_string(&severity).expect("serialization should succeed");
    assert_eq!(json, expected);
}

#[rstest]
#[case("\"ERROR\"", Severity::Error)]
#[case("\"WARNING\"", Severity::Warning)]
#[case("\"INFO\"", Severity::Info)]
fn severity_deserializes_uppercase(#[case] json: &str, #[case] expected: Severity) {
    let severity: Severity = serde_json::from_str(json).expect("deserialization should succeed");
    assert_eq!(severity, expected);
}

#[rstest]
fn severity_rejects_wrong_case() {
    let result = serde_json::from_str::<Severity>("\"Error\"");
    assert!(result.is_err(), "PascalCase 'Error' should be rejected");
}

// ---------------------------------------------------------------------------
// ViolationCategory display
// ---------------------------------------------------------------------------

#[rstest]
#[case(ViolationCategory::Architecture, "clean-architecture")]
#[case(ViolationCategory::Quality, "Quality")]
#[case(ViolationCategory::Organization, "Organization")]
#[case(ViolationCategory::Solid, "SOLID")]
#[case(ViolationCategory::Performance, "Performance")]
#[case(ViolationCategory::Kiss, "KISS")]
#[case(ViolationCategory::Naming, "Naming")]
#[case(ViolationCategory::Refactoring, "Refactoring")]
fn violation_category_display(#[case] category: ViolationCategory, #[case] expected: &str) {
    assert_eq!(category.to_string(), expected);
}

// ---------------------------------------------------------------------------
// NamedCheck and run_checks
// ---------------------------------------------------------------------------

#[rstest]
fn run_checks_collects_violations_from_all_checks() {
    let checks = vec![
        NamedCheck::new("check_a", || Ok(vec![])),
        NamedCheck::new("check_b", || Ok(vec![])),
    ];
    let violations = run_checks("test_validator", checks).expect("should not fail");
    assert!(violations.is_empty());
}

#[rstest]
fn run_checks_propagates_errors() {
    let checks = vec![NamedCheck::new("failing_check", || {
        Err("check failed".into())
    })];
    let result = run_checks("test_validator", checks);
    assert!(result.is_err());
}
