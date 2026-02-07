//! Tests for architecture rules like CA001
//!
//! Uses `CRATE_LAYER_MAPPINGS` from test_constants instead of
//! hardcoded crate names.

use std::path::PathBuf;

use mcb_validate::rules::yaml_loader::YamlRuleLoader;

use crate::test_constants::CRATE_LAYER_MAPPINGS;

#[tokio::test]
async fn test_ca001_rule_loading() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let rules_dir = workspace_root.join("crates/mcb-validate/rules");

    assert!(
        rules_dir.exists(),
        "Rules directory does not exist: {rules_dir:?}"
    );

    let mut variables = serde_yaml::Mapping::new();
    variables.insert(
        serde_yaml::Value::String("project_prefix".to_string()),
        serde_yaml::Value::String("mcb".to_string()),
    );

    for (key, crate_name, module_name) in CRATE_LAYER_MAPPINGS {
        variables.insert(
            serde_yaml::Value::String(format!("{key}_crate")),
            serde_yaml::Value::String(crate_name.to_string()),
        );
        variables.insert(
            serde_yaml::Value::String(format!("{key}_module")),
            serde_yaml::Value::String(module_name.to_string()),
        );
    }

    let mut loader =
        YamlRuleLoader::with_variables(rules_dir, Some(serde_yaml::Value::Mapping(variables)))
            .unwrap();
    let rules = loader.load_all_rules().await.unwrap();

    println!("Loaded {} rules", rules.len());

    let ca001_rule = rules.iter().find(|r| r.id == "CA001");

    if let Some(rule) = ca001_rule {
        println!("Found CA001 rule: {:?}", rule.name);
        assert_eq!(
            rule.engine, "rust-rule-engine",
            "CA001 should use rust-rule-engine"
        );
        assert!(
            rule.name.contains("Domain"),
            "CA001 should be about domain layer"
        );
    } else {
        println!(
            "Available rules: {:?}",
            rules.iter().map(|r| &r.id).collect::<Vec<_>>()
        );
        panic!("CA001 rule should be loaded");
    }
}
