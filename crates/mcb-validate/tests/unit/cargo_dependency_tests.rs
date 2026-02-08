//! Tests for cargo dependency detection.
//!
//! Uses shared helpers:
//! - `create_rule_context` — eliminates duplicated RuleContext construction
//! - `cargo_toml_with_deps` — eliminates inline Cargo.toml template strings

use mcb_validate::engines::hybrid_engine::RuleEngine;
use mcb_validate::engines::rusty_rules_engine::RustyRulesEngineWrapper;
use mcb_validate::ValidationConfig;
use serde_json::json;

use crate::test_constants::*;
use crate::test_utils::*;

#[test]
fn test_cargo_dependency_detection() {
    let engine = RustyRulesEngineWrapper::new();
    let context = create_rule_context();

    let rule_definition = json!({
        "type": "cargo_dependencies",
        "condition": "not_exists",
        "pattern": FORBIDDEN_PREFIX_PATTERN,
        "target": DOMAIN_CRATE
    });

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(engine.execute(&rule_definition, &context));
    assert!(
        result.is_ok(),
        "Cargo dependency rule execution should succeed"
    );

    let violations = result.unwrap();
    assert_eq!(
        violations.len(),
        0,
        "Should not find violations for clean dependencies"
    );
}

#[test]
fn test_cargo_dependency_detection_with_violation() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let cargo_path = temp_dir.path().join("Cargo.toml");

    let cargo_content = cargo_toml_with_deps(
        TEST_SUBJECT_CRATE,
        &[
            ("serde", "1.0"),
            ("my-infrastructure", "0.1.0"),
            (DOMAIN_CRATE, "0.1.0"),
        ],
    );

    std::fs::write(&cargo_path, cargo_content).unwrap();

    let mut context = create_rule_context();
    context.workspace_root = temp_dir.path().to_path_buf();
    context.config = ValidationConfig::new(temp_dir.path());

    let engine = RustyRulesEngineWrapper::new();
    let rule_definition = json!({
        "type": "cargo_dependencies",
        "condition": "not_exists",
        "pattern": FORBIDDEN_PREFIX_PATTERN,
        "target": TEST_SUBJECT_CRATE
    });

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(engine.execute(&rule_definition, &context));

    assert!(
        result.is_ok(),
        "Cargo dependency rule execution should succeed"
    );

    let violations = result.unwrap();

    assert!(
        !violations.is_empty(),
        "Should find violations for forbidden dependencies"
    );
}
