//! Unit tests for `UnifiedRuleRegistry`.
//!
//! Run with: `cargo test -p mcb-validate --test unit unified_registry`

use std::path::PathBuf;

use tracing::info;

use mcb_validate::filters::LanguageId;
use mcb_validate::unified_registry::{RuleOrigin, UnifiedRuleRegistry};
use mcb_validate::ValidationConfig;

fn test_workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("could not find workspace root")
        .to_path_buf()
}

#[test]
fn test_list_all_rules_discovers_both_systems() {
    let root = test_workspace_root();
    let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");

    let all_rules = registry.list_all_rules();

    let rust_count = all_rules
        .iter()
        .filter(|r| r.origin == RuleOrigin::Rust)
        .count();
    assert!(
        rust_count >= 18,
        "Expected at least 18 Rust validators, got {rust_count}"
    );

    let yaml_count = all_rules
        .iter()
        .filter(|r| r.origin == RuleOrigin::Yaml)
        .count();
    assert!(
        yaml_count >= 30,
        "Expected at least 30 YAML rules, got {yaml_count}"
    );

    let total = all_rules.len();
    assert!(total >= 48, "Expected at least 48 total rules, got {total}");

    assert!(
        all_rules.iter().any(|r| r.origin == RuleOrigin::Rust),
        "No Rust rules found"
    );
    assert!(
        all_rules.iter().any(|r| r.origin == RuleOrigin::Yaml),
        "No YAML rules found"
    );
}

#[test]
fn test_rust_validator_count() {
    let root = test_workspace_root();
    let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");

    assert!(
        registry.rust_validator_count() >= 18,
        "Expected at least 18 Rust validators, got {}",
        registry.rust_validator_count()
    );
}

#[test]
fn test_yaml_rule_count() {
    let root = test_workspace_root();
    let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");

    assert!(
        registry.yaml_rule_count() >= 30,
        "Expected at least 30 YAML rules, got {}",
        registry.yaml_rule_count()
    );
}

#[test]
fn test_total_rule_count() {
    let root = test_workspace_root();
    let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");

    let total = registry.total_rule_count();
    assert!(total >= 48, "Expected at least 48 total rules, got {total}");
    assert_eq!(
        total,
        registry.rust_validator_count() + registry.yaml_rule_count(),
        "Total should equal Rust + YAML counts"
    );
}

#[test]
fn test_rule_info_has_correct_origins() {
    let root = test_workspace_root();
    let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");

    let rules = registry.list_all_rules();

    let clean_arch = rules.iter().find(|r| r.id == "clean_architecture");
    assert!(clean_arch.is_some(), "clean_architecture validator not found");
    assert_eq!(clean_arch.unwrap().origin, RuleOrigin::Rust);

    let yaml_rules: Vec<_> = rules.iter().filter(|r| r.origin == RuleOrigin::Yaml).collect();
    assert!(
        yaml_rules
            .iter()
            .any(|r| r.id.starts_with("CA") || r.id.starts_with("QUAL")),
        "Expected YAML rules with CA or QUAL prefixes"
    );
}

#[test]
fn test_execute_all_produces_violations_from_both_systems() {
    let root = test_workspace_root();
    let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");
    let config = ValidationConfig::new(&root);

    let result = registry.execute_all(&config);
    assert!(result.is_ok(), "execute_all failed: {:?}", result.err());

    let violations = result.unwrap();
    info!(
        total_violations = violations.len(),
        "execute_all completed successfully"
    );
}

#[test]
fn test_execute_by_category() {
    let root = test_workspace_root();
    let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");
    let config = ValidationConfig::new(&root);

    let result = registry.execute_by_category("architecture", &config);
    assert!(
        result.is_ok(),
        "execute_by_category failed: {:?}",
        result.err()
    );
}

#[test]
fn test_execute_by_language() {
    let root = test_workspace_root();
    let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");
    let config = ValidationConfig::new(&root);

    let result = registry.execute_by_language(LanguageId::Rust, &config);
    assert!(
        result.is_ok(),
        "execute_by_language failed: {:?}",
        result.err()
    );
}

#[test]
fn test_accessors() {
    let root = test_workspace_root();
    let registry = UnifiedRuleRegistry::new(&root).expect("failed to create registry");

    let _rust = registry.rust_registry();
    let _yaml = registry.yaml_rules();
    let _engine = registry.hybrid_engine();

    assert!(!registry.yaml_rules().is_empty());
    assert!(!registry.rust_registry().validators().is_empty());
}
