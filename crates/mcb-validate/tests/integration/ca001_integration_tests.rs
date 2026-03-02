//! Integration test for CA001 rule.
//!
//! These tests verify the GRL rule logic for detecting forbidden dependencies.
//! Uses the actual workspace so `cargo_metadata` works correctly.

use std::collections::HashMap;

use mcb_validate::ValidationConfig;
use mcb_validate::engines::RuleContext;
use mcb_validate::engines::rete_engine::ReteEngine;
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn test_ca001_detects_mcb_domain_violations() {
    let mut engine = ReteEngine::new();
    let workspace_root = mcb_domain::utils::tests::utils::workspace_root().unwrap();

    let context = RuleContext {
        workspace_root: workspace_root.clone(),
        config: ValidationConfig::new(&workspace_root),
        ast_data: HashMap::new(),
        cargo_data: HashMap::new(),
        file_contents: HashMap::new(),
        facts: std::sync::Arc::new(Vec::new()),
        graph: std::sync::Arc::new(mcb_validate::graph::DependencyGraph::new()),
    };

    let grl = r#"
rule "DomainIndependence" salience 10 {
    when
        Facts.has_internal_dependencies == true
    then
        Facts.violation_triggered = true;
        Facts.violation_message = "Domain layer cannot depend on internal mcb-* crates";
        Facts.violation_rule_name = "CA001";
}
"#;

    let violations = engine
        .execute_grl(grl, &context)
        .await
        .unwrap_or_else(|e| panic!("CA001 rule execution should succeed: {e}"));

    assert!(
        !violations.is_empty(),
        "CA001 should detect internal dependencies in actual workspace"
    );
    assert!(
        violations[0].message.contains("Domain layer cannot depend"),
        "Should have correct violation message"
    );
}

#[rstest]
#[tokio::test]
async fn test_ca001_allows_clean_dependencies() {
    let mut engine = ReteEngine::new();
    let workspace_root = mcb_domain::utils::tests::utils::workspace_root().unwrap();

    let context = RuleContext {
        workspace_root: workspace_root.clone(),
        config: ValidationConfig::new(&workspace_root),
        ast_data: HashMap::new(),
        cargo_data: HashMap::new(),
        file_contents: HashMap::new(),
        facts: std::sync::Arc::new(Vec::new()),
        graph: std::sync::Arc::new(mcb_validate::graph::DependencyGraph::new()),
    };

    let grl = r#"
rule "NoInternalDepsCheck" salience 10 {
    when
        Facts.has_internal_dependencies == false
    then
        Facts.violation_triggered = true;
        Facts.violation_message = "No internal dependencies found";
        Facts.violation_rule_name = "NO_INTERNAL_DEPS";
}
"#;

    let violations = engine
        .execute_grl(grl, &context)
        .await
        .unwrap_or_else(|e| panic!("Rule execution should succeed: {e}"));

    assert!(
        violations.is_empty(),
        "Rule checking for no-internal-deps should NOT fire on a workspace with internal deps"
    );
}
