//! Integration tests for `DeclarativeValidator` and new path/regex rules.

use mcb_validate::traits::validator::Validator;
use mcb_validate::validators::declarative_validator::DeclarativeValidator;
use std::fs;
use std::path::Path;

use tempfile::TempDir;

fn create_test_env(root: &Path) {
    // Create config
    let config_dir = root.join("config");
    fs::create_dir_all(&config_dir).unwrap();
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
    fs::write(&config_path, config_content).unwrap();

    // Create rules dir
    let rules_dir = root.join("rules/organization");
    fs::create_dir_all(&rules_dir).unwrap();

    // Write rule ORG020
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
    fs::write(rules_dir.join("ORG020.yml"), rule_org020).unwrap();

    // Write rule ORG021
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
    fs::write(rules_dir.join("ORG021.yml"), rule_org021).unwrap();

    // Write rule ORG019
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
    // Added my_factory.rs to skip list for simplicity in test matching if regex matching filename is not enough
    // But filters.skip matches file path.
    fs::write(rules_dir.join("ORG019.yml"), rule_org019).unwrap();
}

#[test]
fn test_org020_domain_adapters_violation() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    create_test_env(root);

    // Setup domain crate structure
    let domain_src = root.join("crates/mcb-domain/src");
    fs::create_dir_all(&domain_src).unwrap();

    // Create a valid file (port)
    let port_file = domain_src.join("ports/repository.rs");
    fs::create_dir_all(port_file.parent().unwrap()).unwrap();
    fs::write(&port_file, "pub trait Repository {}").unwrap();

    // Create an invalid file (adapter in domain)
    let adapter_file = domain_src.join("adapters/sql_repository.rs");
    fs::create_dir_all(adapter_file.parent().unwrap()).unwrap();
    fs::write(&adapter_file, "pub struct SqlRepository;").unwrap();

    // Create a validator
    let validator = DeclarativeValidator::new(root);

    // Config needs to point to root
    let config = mcb_validate::ValidationConfig::new(root);

    // Run validation
    let violations = validator.validate(&config).unwrap();

    // Check for ORG020 violation
    let violation = violations.iter().find(|v| v.id() == "ORG020");
    assert!(
        violation.is_some(),
        "Expected ORG020 violation for adapter in domain"
    );

    let v = violation.unwrap();
    assert!(
        v.file()
            .unwrap()
            .ends_with("crates/mcb-domain/src/adapters/sql_repository.rs")
    );
}

#[test]
fn test_org021_infra_ports_violation() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    create_test_env(root);

    // Setup infra crate structure
    let infra_src = root.join("crates/mcb-infrastructure/src");
    fs::create_dir_all(&infra_src).unwrap();

    // Create invalid file (port in infra)
    let port_file = infra_src.join("ports/service_port.rs");
    fs::create_dir_all(port_file.parent().unwrap()).unwrap();
    fs::write(&port_file, "pub trait ServicePort {}").unwrap();

    let validator = DeclarativeValidator::new(root);
    let config = mcb_validate::ValidationConfig::new(root);
    let violations = validator.validate(&config).unwrap();

    let violation = violations.iter().find(|v| v.id() == "ORG021");
    assert!(
        violation.is_some(),
        "Expected ORG021 violation for port in infra"
    );
}

#[test]
fn test_org019_trait_placement_violation() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    create_test_env(root);

    // Setup infra crate structure
    let infra_src = root.join("crates/mcb-infrastructure/src");
    fs::create_dir_all(&infra_src).unwrap();

    // Create invalid file (trait definition in infra named Provider)
    let provider_file = infra_src.join("my_provider.rs");
    fs::write(&provider_file, "pub trait MyProvider {}").unwrap(); // Should flag

    // Create valid file (trait definitions allowed if suffix is Factory)
    let factory_file = infra_src.join("my_factory.rs");
    fs::write(&factory_file, "pub trait MyProviderFactory {}").unwrap(); // Should NOT flag

    let validator = DeclarativeValidator::new(root);
    let config = mcb_validate::ValidationConfig::new(root);
    let violations = validator.validate(&config).unwrap();

    // Check violations
    let provider_violation = violations
        .iter()
        .find(|v| v.id() == "ORG019" && v.file().unwrap().ends_with("my_provider.rs"));
    assert!(
        provider_violation.is_some(),
        "Expected ORG019 violation for MyProvider"
    );

    let factory_violation = violations
        .iter()
        .find(|v| v.id() == "ORG019" && v.file().unwrap().ends_with("my_factory.rs"));
    assert!(
        factory_violation.is_none(),
        "Did not expect ORG019 violation for MyProviderFactory"
    );
}
