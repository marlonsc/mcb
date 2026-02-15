//! Integration tests for DeclarativeValidator and new path/regex rules.

use mcb_validate::traits::validator::Validator;
use mcb_validate::validators::declarative_validator::DeclarativeValidator;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_org020_domain_adapters_violation() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

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
