//! Integration tests for `DeclarativeValidator` and new path/regex rules.

use std::fs;
use std::io;
use std::path::Path;

use mcb_validate::ValidationConfig;
use mcb_validate::ValidationError;
use mcb_validate::traits::validator::Validator;
use mcb_validate::validators::declarative_validator::DeclarativeValidator;
use rstest::rstest;
use tempfile::TempDir;

fn create_test_env(root: &Path) -> io::Result<()> {
    let config_dir = root.join("config");
    fs::create_dir_all(&config_dir)?;
    let config_path = config_dir.join("mcb-validate-internal.toml");
    let config_content = r#"
[general]
rules_path = "rules"

[rules.clean_architecture]
infrastructure_path = "crates/mcb-infrastructure/src"
domain_path = "crates/mcb-domain/src"
server_path = "crates/mcb-server/src"
application_path = "crates/mcb-infrastructure/src"

[rules.naming]
domain_crate = "mcb-domain"
infrastructure_crate = "mcb-infrastructure"
server_crate = "mcb-server"
application_crate = "mcb-infrastructure"
"#;
    fs::write(&config_path, config_content)?;

    let rules_dir = root.join("rules/organization");
    fs::create_dir_all(&rules_dir)?;

    let rule_org020 = r#"
schema: "rule/v2"
id: ORG020
name: Domain Adapters
category: organization
severity: error
description: Adapters belong in the infrastructure layer.
rationale: Domain layer should be pure.
engine: path
rule:
  type: file_location
filters:
  file_patterns:
    - "{{ domain_path }}/**/adapters/**/*.rs"
"#;
    fs::write(rules_dir.join("ORG020.yml"), rule_org020)?;

    let rule_org021 = r#"
schema: "rule/v2"
id: ORG021
name: Infrastructure Ports
category: organization
severity: error
description: Port definitions belong in the domain layer.
rationale: Infrastructure layer should implement ports.
engine: path
rule:
  type: file_location
filters:
  file_patterns:
    - "{{ infrastructure_path }}/**/ports/**/*.rs"
"#;
    fs::write(rules_dir.join("ORG021.yml"), rule_org021)?;

    let rule_org019 = r#"
schema: "rule/v2"
id: ORG019
name: Infra Trait Location
category: organization
severity: warning
description: Traits in infra should be factories or specific types.
rationale: Core traits belong in domain.
engine: regex
rule:
  type: regex_scan
config:
  patterns:
    trait_def: 'pub\s+trait\s+([A-Z][a-zA-Z0-9_]*)'
filters:
  file_patterns:
    - "crates/{{ infrastructure_crate }}/**/*.rs"
  skip:
    file_patterns:
      - "**/*Factory.rs"
      - "**/my_factory.rs"
"#;
    fs::write(rules_dir.join("ORG019.yml"), rule_org019)?;
    Ok(())
}

#[rstest]
#[test]
fn test_org020_domain_adapters_violation() -> io::Result<()> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();
    create_test_env(root)?;

    let domain_src = root.join("crates/mcb-domain/src");
    fs::create_dir_all(&domain_src)?;

    let port_file = domain_src.join("ports/repository.rs");
    if let Some(parent) = port_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&port_file, "pub trait Repository {}")?;

    let adapter_file = domain_src.join("adapters/sql_repository.rs");
    if let Some(parent) = adapter_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&adapter_file, "pub struct SqlRepository;")?;

    let validator = DeclarativeValidator::new(root);
    let config = ValidationConfig::new(root);
    let violations = validator
        .validate(&config)
        .map_err(|e| io::Error::other(e.to_string()))?;

    let violation = violations
        .iter()
        .find(|v| v.id() == "ORG020")
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Expected ORG020 violation for adapter in domain",
            )
        })?;
    let path = violation
        .file()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "violation has no file"))?;
    assert!(path.ends_with("crates/mcb-domain/src/adapters/sql_repository.rs"));
    Ok(())
}

#[rstest]
#[test]
fn test_org021_infra_ports_violation() -> io::Result<()> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();
    create_test_env(root)?;

    let infra_src = root.join("crates/mcb-infrastructure/src");
    fs::create_dir_all(&infra_src)?;

    let port_file = infra_src.join("ports/service_port.rs");
    if let Some(parent) = port_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&port_file, "pub trait ServicePort {}")?;

    let validator = DeclarativeValidator::new(root);
    let config = ValidationConfig::new(root);
    let violations = validator
        .validate(&config)
        .map_err(|e| io::Error::other(e.to_string()))?;

    let violation = violations
        .iter()
        .find(|v| v.id() == "ORG021")
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Expected ORG021 violation for port in infra",
            )
        })?;
    let _ = violation;
    Ok(())
}

#[rstest]
#[test]
fn test_org019_trait_placement_violation() -> io::Result<()> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();
    create_test_env(root)?;

    let infra_src = root.join("crates/mcb-infrastructure/src");
    fs::create_dir_all(&infra_src)?;

    let provider_file = infra_src.join("my_provider.rs");
    fs::write(&provider_file, "pub trait MyProvider {}")?;

    let factory_file = infra_src.join("my_factory.rs");
    fs::write(&factory_file, "pub trait MyProviderFactory {}")?;

    let validator = DeclarativeValidator::new(root);
    let config = ValidationConfig::new(root);
    let violations = validator
        .validate(&config)
        .map_err(|e| io::Error::other(e.to_string()))?;

    let provider_violation = violations
        .iter()
        .find(|v| v.id() == "ORG019" && v.file().is_some_and(|p| p.ends_with("my_provider.rs")));
    assert!(
        provider_violation.is_some(),
        "Expected ORG019 violation for MyProvider"
    );

    let factory_violation = violations
        .iter()
        .find(|v| v.id() == "ORG019" && v.file().is_some_and(|p| p.ends_with("my_factory.rs")));
    assert!(
        factory_violation.is_none(),
        "Did not expect ORG019 violation for MyProviderFactory"
    );
    Ok(())
}

fn write_ast_rule(
    root: &Path,
    rule_id: &str,
    ast_query: &str,
    include_selector: bool,
) -> io::Result<()> {
    let rules_dir = root.join("rules/quality");
    fs::create_dir_all(&rules_dir)?;

    let selectors = if include_selector {
        "
selectors:
  - language: rust
    node_type: function_item
"
    } else {
        ""
    };

    let rule = format!(
        r#"
schema: "rule/v2"
id: {rule_id}
name: AST Query Rule
category: quality
severity: warning
description: Verify AST query execution.
rationale: Declarative AST query must execute.
engine: regex
rule:
  type: regex_scan
config:
  patterns:
    placeholder: 'fn\\s+'
message: "AST violation"
{selectors}
ast_query: "{ast_query}"
filters:
  file_patterns:
    - "crates/mcb-domain/**/*.rs"
"#,
    );

    fs::write(rules_dir.join(format!("{rule_id}.yml")), rule)?;
    Ok(())
}

#[rstest]
#[test]
fn test_ast_query_and_selector_execute_together() -> io::Result<()> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();
    create_test_env(root)?;

    write_ast_rule(
        root,
        "AST001",
        "(function_item name: (identifier) @name)",
        true,
    )?;

    let source_file = root.join("crates/mcb-domain/src/sample.rs");
    if let Some(parent) = source_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&source_file, "fn one() {}\nfn two() {}\nfn three() {}\n")?;

    let validator = DeclarativeValidator::new(root);
    let config = ValidationConfig::new(root);
    let violations = validator
        .validate(&config)
        .map_err(|e| io::Error::other(e.to_string()))?;

    let ast_count = violations.iter().filter(|v| v.id() == "AST001").count();
    assert_eq!(
        ast_count, 6,
        "expected selector and ast_query to both produce 3 matches"
    );
    Ok(())
}

#[rstest]
#[test]
fn test_invalid_ast_query_returns_config_error() -> io::Result<()> {
    let temp_dir = TempDir::new()?;
    let root = temp_dir.path();
    create_test_env(root)?;

    write_ast_rule(
        root,
        "AST002",
        "(function_item name: (identifier) @name",
        false,
    )?;

    let source_file = root.join("crates/mcb-domain/src/sample.rs");
    if let Some(parent) = source_file.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&source_file, "fn bad() {}\n")?;

    let validator = DeclarativeValidator::new(root);
    let config = ValidationConfig::new(root);
    let result = validator.validate(&config);

    match result {
        Err(ValidationError::Config(message)) => {
            assert!(message.contains("Invalid tree-sitter query"));
        }
        Err(other) => panic!("expected ValidationError::Config, got {other:?}"),
        Ok(_) => panic!("expected validation to fail for invalid tree-sitter query"),
    }
    Ok(())
}
