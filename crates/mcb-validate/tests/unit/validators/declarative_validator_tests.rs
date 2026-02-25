//! Unit tests for `DeclarativeValidator` (YAML-driven rules with AST selectors).

use std::fs;
use std::io;
use std::path::Path;

use mcb_validate::ValidationConfig;
use mcb_validate::traits::validator::Validator;
use mcb_validate::validators::declarative_validator::DeclarativeValidator;
use tempfile::TempDir;

fn write_workspace_manifest(root: &Path) -> io::Result<()> {
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/demo\"]\n",
    )?;
    Ok(())
}

fn write_validator_config(root: &Path) -> io::Result<()> {
    let config_dir = root.join("config");
    fs::create_dir_all(&config_dir)?;
    fs::write(
        config_dir.join("mcb-validate-internal.toml"),
        "[general]\nrules_path = \"rules\"\n",
    )?;
    Ok(())
}

fn write_source_file(root: &Path, content: &str) -> io::Result<()> {
    let src_dir = root.join("crates/demo/src");
    fs::create_dir_all(&src_dir)?;
    fs::write(src_dir.join("lib.rs"), content)?;
    Ok(())
}

fn write_rule(root: &Path, file_name: &str, content: &str) -> io::Result<()> {
    let rules_dir = root.join("rules");
    fs::create_dir_all(&rules_dir)?;
    fs::write(rules_dir.join(file_name), content)?;
    Ok(())
}

#[test]
fn ast_selector_matches_rust_function_nodes() -> io::Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path();
    write_workspace_manifest(root)?;
    write_validator_config(root)?;
    write_source_file(root, "pub fn first() {}\n\nfn second() { let _x = 1; }\n")?;
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
    )?;

    let validator = DeclarativeValidator::new(root);
    let config = ValidationConfig::new(root);
    let violations = validator
        .validate(&config)
        .map_err(|e| io::Error::other(e.to_string()))?;

    let ast_count = violations.iter().filter(|v| v.id() == "AST001").count();
    assert!(
        ast_count >= 2,
        "expected AST selector rule to match both functions"
    );
    Ok(())
}

#[test]
fn regex_rules_without_selectors_still_work() -> io::Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path();
    write_workspace_manifest(root)?;
    write_validator_config(root)?;
    write_source_file(
        root,
        "pub fn uses_unwrap(v: Option<u8>) -> u8 { v.unwrap() }\n",
    )?;
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
    )?;

    let validator = DeclarativeValidator::new(root);
    let config = ValidationConfig::new(root);
    let violations = validator
        .validate(&config)
        .map_err(|e| io::Error::other(e.to_string()))?;

    assert!(
        violations.iter().any(|v| v.id() == "REG001"),
        "expected regex rule to execute when selectors are absent"
    );
    Ok(())
}
