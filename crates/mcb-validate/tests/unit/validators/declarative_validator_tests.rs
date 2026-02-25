use std::fs;
use std::path::Path;

use mcb_validate::ValidationConfig;
use mcb_validate::traits::validator::Validator;
use mcb_validate::validators::declarative_validator::DeclarativeValidator;
use tempfile::TempDir;

fn write_workspace_manifest(root: &Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/demo\"]\n",
    )
    .unwrap();
}

fn write_validator_config(root: &Path) {
    let config_dir = root.join("config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("mcb-validate-internal.toml"),
        "[general]\nrules_path = \"rules\"\n",
    )
    .unwrap();
}

fn write_source_file(root: &Path, content: &str) {
    let src_dir = root.join("crates/demo/src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), content).unwrap();
}

fn write_rule(root: &Path, file_name: &str, content: &str) {
    let rules_dir = root.join("rules");
    fs::create_dir_all(&rules_dir).unwrap();
    fs::write(rules_dir.join(file_name), content).unwrap();
}

#[test]
fn ast_selector_matches_rust_function_nodes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_workspace_manifest(root);
    write_validator_config(root);
    write_source_file(root, "pub fn first() {}\n\nfn second() { let _x = 1; }\n");
    write_rule(
        root,
        "AST001.yml",
        r#"
schema: "rule/v2"
id: "AST001"
name: "Match Rust functions"
category: "quality"
severity: "warning"
enabled: true
description: "Function declarations should be detected"
rationale: "Coverage for AST selector execution"
engine: "regex"
selectors:
  - language: "rust"
    node_type: "function_item"
message: "Function node detected"
"#,
    );

    let validator = DeclarativeValidator::new(root);
    let config = ValidationConfig::new(root);
    let violations = validator.validate(&config).unwrap();

    let ast_violations: Vec<_> = violations.iter().filter(|v| v.id() == "AST001").collect();
    assert!(
        ast_violations.len() >= 2,
        "expected AST selector rule to match both functions"
    );
}

#[test]
fn regex_rules_without_selectors_still_work() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();
    write_workspace_manifest(root);
    write_validator_config(root);
    write_source_file(
        root,
        "pub fn uses_unwrap(v: Option<u8>) -> u8 { v.unwrap() }\n",
    );
    write_rule(
        root,
        "REG001.yml",
        r#"
schema: "rule/v2"
id: "REG001"
name: "Regex fallback"
category: "quality"
severity: "error"
enabled: true
description: "Regex-only rules must keep executing"
rationale: "No regression for non-selector rules"
engine: "regex"
rule:
  type: regex_scan
config:
  patterns:
    unwrap_call: "\\.unwrap\\(\\)"
message: "Unwrap detected"
"#,
    );

    let validator = DeclarativeValidator::new(root);
    let config = ValidationConfig::new(root);
    let violations = validator.validate(&config).unwrap();

    assert!(
        violations.iter().any(|v| v.id() == "REG001"),
        "expected regex rule to execute when selectors are absent"
    );
}
