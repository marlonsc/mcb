use mcb_domain::utils::tests::utils::TestResult;
use mcb_validate::rules::templates::TemplateEngine;
use rstest::rstest;

#[rstest]
#[tokio::test]
async fn test_apply_cargo_dependency_template() -> TestResult {
    let mut engine = TemplateEngine::new();
    let temp_dir = tempfile::TempDir::new()?;

    let templates_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&templates_dir)?;

    let template_content = r#"
schema: "template/v1"
_base: true
name: "cargo_dependency_check"
description: "Template for checking Cargo.toml dependencies"
category: "architecture"
severity: "error"
enabled: true

config:
  crate_name: "{{crate_name}}"
  forbidden_prefixes: {{forbidden_prefixes}}
  allowed_dependencies: {{allowed_dependencies}}

validation:
  fields:
    crate_name: ["length(min=1)"]
    forbidden_prefixes: ["length(min=1)"]

rule:
  type: "cargo_dependencies"
  condition: "not_exists"
  pattern: "{{forbidden_prefixes}}"
  target: "{{crate_name}}"
  message: "Crate '{{crate_name}}' must not depend on {{forbidden_prefixes}}"

fixes:
  - type: "remove_dependencies"
    pattern: "{{forbidden_prefixes}}"
    message: "Remove forbidden dependencies from {{crate_name}}/Cargo.toml"
"#;

    std::fs::write(
        templates_dir.join("cargo-dependency-check.yml"),
        template_content,
    )?;

    engine.load_templates(temp_dir.path()).await?;

    let rule_yaml: serde_yaml::Value = serde_yaml::from_str(
        r#"
_template: "cargo_dependency_check"
id: "CA001"
name: "Domain Layer Independence"
category: "architecture"
severity: "error"
enabled: true
engine: "rust-rule-engine"

description: "Domain crate must have zero internal mcb-* dependencies"
rationale: "Domain layer contains pure business logic independent of frameworks"

crate_name: "mcb-domain"
forbidden_prefixes: ["mcb-"]
allowed_dependencies: ["std", "serde", "thiserror", "uuid", "chrono"]

config:
  crate_name: "mcb-domain"
  forbidden_prefixes: ["mcb-"]
  allowed_dependencies: ["std", "serde", "thiserror", "uuid", "chrono"]

rule: |
  rule "DomainIndependence" salience 10 {
      when
          Facts.has_internal_dependencies == true
      then
          Facts.violation_triggered = true;
          Facts.violation_message = "Domain layer cannot depend on internal mcb-* crates";
          Facts.violation_rule_name = "CA001";
  }
"#,
    )?;

    let result = engine.apply_template("cargo_dependency_check", &rule_yaml)?;

    assert_eq!(result["id"], "CA001");
    assert_eq!(result["name"], "Domain Layer Independence");
    assert_eq!(result["category"], "architecture");
    assert!(
        result.get("id").is_some(),
        "Template should preserve rule ID"
    );
    assert!(
        result.get("name").is_some(),
        "Template should preserve rule name"
    );
    assert!(
        result.get("category").is_some(),
        "Template should preserve category"
    );
    Ok(())
}
