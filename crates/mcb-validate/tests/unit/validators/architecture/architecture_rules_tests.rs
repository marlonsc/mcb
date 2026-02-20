//! Tests for architecture rules like CA001.
//!
//! Uses `build_yaml_variables` from shared helpers to eliminate the
//! 15-line YAML variable building boilerplate, and `get_workspace_root`
//! instead of hardcoded paths.

use mcb_validate::rules::yaml_loader::YamlRuleLoader;
use rstest::*;

use crate::utils::test_constants::*;
use crate::utils::{build_yaml_variables, get_workspace_root};

#[rstest]
#[tokio::test]
async fn test_ca001_rule_loading() {
    let workspace_root = get_workspace_root();
    let rules_dir = workspace_root.join("crates/mcb-validate/rules");

    assert!(
        rules_dir.exists(),
        "Rules directory does not exist: {rules_dir:?}"
    );

    let variables = build_yaml_variables();
    let mut loader = YamlRuleLoader::with_variables(rules_dir, Some(variables)).unwrap();
    let rules = loader.load_all_rules().await.unwrap();

    println!("Loaded {} rules", rules.len());

    let ca001_rule = rules.iter().find(|r| r.id == RULE_CA001);

    if let Some(rule) = ca001_rule {
        println!("Found CA001 rule: {:?}", rule.name);
        assert_eq!(
            rule.engine, ENGINE_RUST_RULE,
            "CA001 should use {ENGINE_RUST_RULE}"
        );
        assert!(
            rule.name.contains(RULE_CA001_NAME_KEYWORD),
            "CA001 should be about domain layer"
        );
    } else {
        println!(
            "Available rules: {:?}",
            rules.iter().map(|r| &r.id).collect::<Vec<_>>()
        );
        panic!("{RULE_CA001} rule should be loaded");
    }
}
