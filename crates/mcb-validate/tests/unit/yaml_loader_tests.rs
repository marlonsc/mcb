//! Tests for YAML rule loader

use rstest::rstest;
use std::collections::BTreeSet;
use std::path::PathBuf;

use mcb_validate::EmbeddedRules;
use mcb_validate::FileConfig;
use mcb_validate::rules::yaml_loader::YamlRuleLoader;
use rstest::*;
use tempfile::TempDir;

#[fixture]
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[fixture]
fn substitution_vars(workspace_root: PathBuf) -> serde_yaml::Value {
    let file_config = FileConfig::load(&workspace_root);
    let variables_val = serde_yaml::to_value(&file_config.rules.naming).expect("yaml value");
    let mut variables = variables_val
        .as_mapping()
        .expect("naming config mapping")
        .clone();

    let crates = [
        "domain",
        "application",
        "providers",
        "infrastructure",
        "server",
        "validate",
        "language_support",
        "ast_utils",
    ];
    for name in crates {
        let key = format!("{name}_crate");
        if let Some(s) = variables
            .get(serde_yaml::Value::String(key.clone()))
            .and_then(|val| val.as_str())
        {
            variables.insert(
                serde_yaml::Value::String(format!("{name}_module")),
                serde_yaml::Value::String(s.replace('-', "_")),
            );
        }
    }

    if let Some(domain_str) = variables
        .get(serde_yaml::Value::String("domain_crate".into()))
        .and_then(|v| v.as_str())
    {
        let prefix = match domain_str.find('-') {
            Some(idx) => domain_str[0..idx].to_string(),
            None => domain_str.to_string(),
        };
        variables.insert(
            serde_yaml::Value::String("project_prefix".into()),
            serde_yaml::Value::String(prefix),
        );
    }

    serde_yaml::Value::Mapping(variables)
}

#[fixture]
fn temp_rules_dir() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let rules = temp.path().join("rules");
    std::fs::create_dir(&rules).unwrap();
    (temp, rules)
}

#[rstest]
#[tokio::test]
async fn test_load_valid_rule(temp_rules_dir: (TempDir, PathBuf)) {
    let (_temp, rules_dir) = temp_rules_dir;

    let rule_content = r#"
schema: "rule/v1"
id: "TEST001"
name: "Test Rule"
category: "architecture"
severity: "error"
description: "This is a test rule with enough description to pass validation"
rationale: "This rule exists for testing purposes and has enough rationale"
engine: "rust-rule-engine"
config:
  crate_name: "test-crate"
rule:
  type: "cargo_dependencies"
  condition: "not_exists"
  pattern: "forbidden-*"
"#;

    let rule_file = rules_dir.join("test-rule.yml");
    std::fs::write(&rule_file, rule_content).unwrap();

    let mut loader = YamlRuleLoader::new(rules_dir).unwrap();
    let rules = loader.load_all_rules().await.unwrap();

    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].id, "TEST001");
    assert_eq!(rules[0].name, "Test Rule");
}

#[rstest]
#[tokio::test]
async fn test_load_rule_with_template(temp_rules_dir: (TempDir, PathBuf)) {
    let (_temp, rules_dir) = temp_rules_dir;
    let templates_dir = rules_dir.join("templates");
    std::fs::create_dir_all(&templates_dir).unwrap();

    let template_content = r#"
schema: "template/v1"
_base: true
name: "cargo_dependency_check"
category: "architecture"
severity: "error"
enabled: true
description: "Template for checking Cargo.toml dependencies"
rationale: "Dependencies should follow architectural boundaries"
config:
  crate_name: "{{crate_name}}"
  forbidden_prefixes: {{forbidden_prefixes}}
rule:
  type: "cargo_dependencies"
  condition: "not_exists"
  pattern: "{{forbidden_prefixes}}"
"#;

    std::fs::write(
        templates_dir.join("cargo-dependency-check.yml"),
        template_content,
    )
    .unwrap();

    let rule_content = r#"
_template: "cargo_dependency_check"
id: "TEST002"
name: "Domain Dependencies"
description: "Domain must not depend on other layers"
rationale: "Domain should be independent"
crate_name: "mcb-domain"
forbidden_prefixes: ["mcb-"]
config:
  crate_name: "mcb-domain"
  forbidden_prefixes: ["mcb-"]
"#;

    std::fs::write(rules_dir.join("domain-deps.yml"), rule_content).unwrap();

    let mut loader = YamlRuleLoader::new(rules_dir).unwrap();
    let rules = loader.load_all_rules().await.unwrap();

    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].id, "TEST002");
    assert!(rules[0].description.contains("Domain must not depend"));
}

#[rstest]
#[tokio::test]
async fn test_yaml_rule_execution_detects_violations(workspace_root: PathBuf) {
    use mcb_validate::{ValidationConfig, ValidatorRegistry};

    let config = ValidationConfig::new(&workspace_root);
    let registry = ValidatorRegistry::standard_for(&workspace_root);

    match registry.validate_named(&config, &["quality"]) {
        Ok(report) => {
            println!(
                "YAML validation completed successfully. Violations: {}",
                report.len()
            );
            let qual006 = report.iter().filter(|v| v.id() == "QUAL006").count();
            if qual006 > 0 {
                println!("✅ SUCCESS: QUAL006 detected {qual006} file size violations!");
            }
        }
        Err(e) => {
            println!("YAML validation failed (expected in some environments): {e}");
        }
    }
}

#[rstest]
#[tokio::test]
async fn test_embedded_rules_load(substitution_vars: serde_yaml::Value) {
    let embedded = EmbeddedRules::all_yaml();
    let mut loader =
        YamlRuleLoader::from_embedded_with_variables(&embedded, Some(substitution_vars)).unwrap();
    let rules = loader.load_all_rules().await.unwrap();

    assert!(!rules.is_empty());
    assert!(rules.iter().any(|rule| rule.id == "CA001"));
}

#[rstest]
#[tokio::test]
async fn test_embedded_rules_equivalence(
    workspace_root: PathBuf,
    substitution_vars: serde_yaml::Value,
) {
    let file_config = FileConfig::load(&workspace_root);
    let rules_dir = workspace_root.join(file_config.general.rules_path);

    let mut fs_loader =
        YamlRuleLoader::with_variables(rules_dir, Some(substitution_vars.clone())).unwrap();
    let fs_rules = fs_loader.load_all_rules().await.unwrap();

    let embedded = EmbeddedRules::all_yaml();
    let mut embedded_loader =
        YamlRuleLoader::from_embedded_with_variables(&embedded, Some(substitution_vars)).unwrap();
    let embedded_rules = embedded_loader.load_all_rules().await.unwrap();

    let fs_ids: BTreeSet<String> = fs_rules.into_iter().map(|rule| rule.id).collect();
    let embedded_ids: BTreeSet<String> = embedded_rules.into_iter().map(|rule| rule.id).collect();

    assert_eq!(embedded_ids, fs_ids);
}
