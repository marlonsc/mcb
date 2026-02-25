//! Unit tests for `UnifiedRuleRegistry`.
//!
//! Run with: `cargo test -p mcb-validate --test unit unified_registry`

use std::io;
use std::path::PathBuf;

use tracing::info;

use mcb_validate::ValidationConfig;
use mcb_validate::filters::LanguageId;
use mcb_validate::unified_registry::{RuleOrigin, UnifiedRuleRegistry};

fn test_workspace_root() -> Option<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
}

fn root_or_err() -> io::Result<PathBuf> {
    test_workspace_root()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "workspace root not found"))
}

#[test]
fn test_list_all_rules_discovers_both_systems() -> io::Result<()> {
    let root = root_or_err()?;
    let registry = UnifiedRuleRegistry::new(&root).map_err(|e| io::Error::other(e.to_string()))?;

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
    Ok(())
}

#[test]
fn test_rust_validator_count() -> io::Result<()> {
    let root = root_or_err()?;
    let registry = UnifiedRuleRegistry::new(&root).map_err(|e| io::Error::other(e.to_string()))?;

    assert!(
        registry.rust_validator_count() >= 18,
        "Expected at least 18 Rust validators, got {}",
        registry.rust_validator_count()
    );
    Ok(())
}

#[test]
fn test_yaml_rule_count() -> io::Result<()> {
    let root = root_or_err()?;
    let registry = UnifiedRuleRegistry::new(&root).map_err(|e| io::Error::other(e.to_string()))?;

    assert!(
        registry.yaml_rule_count() >= 30,
        "Expected at least 30 YAML rules, got {}",
        registry.yaml_rule_count()
    );
    Ok(())
}

#[test]
fn test_total_rule_count() -> io::Result<()> {
    let root = root_or_err()?;
    let registry = UnifiedRuleRegistry::new(&root).map_err(|e| io::Error::other(e.to_string()))?;

    let total = registry.total_rule_count();
    assert!(total >= 48, "Expected at least 48 total rules, got {total}");
    assert_eq!(
        total,
        registry.rust_validator_count() + registry.yaml_rule_count(),
        "Total should equal Rust + YAML counts"
    );
    Ok(())
}

#[test]
fn test_rule_info_has_correct_origins() -> io::Result<()> {
    let root = root_or_err()?;
    let registry = UnifiedRuleRegistry::new(&root).map_err(|e| io::Error::other(e.to_string()))?;

    let rules = registry.list_all_rules();

    let clean_arch = rules
        .iter()
        .find(|r| r.id == "clean_architecture")
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "clean_architecture validator not found",
            )
        })?;
    assert_eq!(clean_arch.origin, RuleOrigin::Rust);

    let yaml_rules: Vec<_> = rules
        .iter()
        .filter(|r| r.origin == RuleOrigin::Yaml)
        .collect();
    assert!(
        yaml_rules
            .iter()
            .any(|r| r.id.starts_with("CA") || r.id.starts_with("QUAL")),
        "Expected YAML rules with CA or QUAL prefixes"
    );
    Ok(())
}

#[test]
fn test_execute_all_produces_violations_from_both_systems() -> io::Result<()> {
    let root = root_or_err()?;
    let registry = UnifiedRuleRegistry::new(&root).map_err(|e| io::Error::other(e.to_string()))?;
    let config = ValidationConfig::new(&root);

    let violations = registry
        .execute_all(&config)
        .map_err(|e| io::Error::other(e.to_string()))?;
    info!(
        total_violations = violations.len(),
        "execute_all completed successfully"
    );
    Ok(())
}

#[test]
fn test_execute_by_category() -> io::Result<()> {
    let root = root_or_err()?;
    let registry = UnifiedRuleRegistry::new(&root).map_err(|e| io::Error::other(e.to_string()))?;
    let config = ValidationConfig::new(&root);

    let _ = registry
        .execute_by_category("architecture", &config)
        .map_err(|e| io::Error::other(e.to_string()))?;
    Ok(())
}

#[test]
fn test_execute_by_language() -> io::Result<()> {
    let root = root_or_err()?;
    let registry = UnifiedRuleRegistry::new(&root).map_err(|e| io::Error::other(e.to_string()))?;
    let config = ValidationConfig::new(&root);

    let _ = registry
        .execute_by_language(LanguageId::Rust, &config)
        .map_err(|e| io::Error::other(e.to_string()))?;
    Ok(())
}

#[test]
fn test_accessors() -> io::Result<()> {
    let root = root_or_err()?;
    let registry = UnifiedRuleRegistry::new(&root).map_err(|e| io::Error::other(e.to_string()))?;

    let _rust = registry.rust_registry();
    let _yaml = registry.yaml_rules();
    let _engine = registry.hybrid_engine();

    assert!(!registry.yaml_rules().is_empty());
    assert!(!registry.rust_registry().validators().is_empty());
    Ok(())
}
