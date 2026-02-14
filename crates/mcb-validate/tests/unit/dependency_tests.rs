use std::fs;

use mcb_validate::{DependencyValidator, DependencyViolation};
use rstest::*;
use tempfile::TempDir;

fn create_test_workspace() -> TempDir {
    let temp = TempDir::new().unwrap();

    fs::write(
        temp.path().join("Cargo.toml"),
        r#"
[workspace]
members = ["crates/mcb-domain", "crates/mcb-infrastructure"]
"#,
    )
    .unwrap();

    let domain_dir = temp.path().join("crates/mcb-domain");
    fs::create_dir_all(domain_dir.join("src")).unwrap();
    fs::write(
        domain_dir.join("Cargo.toml"),
        r#"
[package]
name = "mcb-domain"
version = "0.1.1"

[dependencies]
serde = "1.0"
"#,
    )
    .unwrap();
    fs::write(domain_dir.join("src/lib.rs"), "pub fn domain() {}").unwrap();

    let infra_dir = temp.path().join("crates/mcb-infrastructure");
    fs::create_dir_all(infra_dir.join("src")).unwrap();
    fs::write(
        infra_dir.join("Cargo.toml"),
        r#"
[package]
name = "mcb-infrastructure"
version = "0.1.1"

[dependencies]
mcb-domain = { path = "../mcb-domain" }
"#,
    )
    .unwrap();
    fs::write(
        infra_dir.join("src/lib.rs"),
        "use mcb_domain::domain;\npub fn infra() { domain(); }",
    )
    .unwrap();

    temp
}

#[rstest]
fn test_valid_dependencies() {
    let temp = create_test_workspace();
    let validator = DependencyValidator::new(temp.path());

    let violations = validator.validate_cargo_dependencies().unwrap();
    assert!(
        violations.is_empty(),
        "Expected no violations, got: {violations:?}"
    );
}

#[rstest]
fn test_forbidden_dependency() {
    let temp = TempDir::new().unwrap();

    let domain_dir = temp.path().join("crates/mcb-domain");
    fs::create_dir_all(domain_dir.join("src")).unwrap();
    fs::write(
        domain_dir.join("Cargo.toml"),
        r#"
[package]
name = "mcb-domain"
version = "0.1.1"

[dependencies]
mcb-infrastructure = { path = "../mcb-infrastructure" }
"#,
    )
    .unwrap();
    fs::write(domain_dir.join("src/lib.rs"), "").unwrap();

    let validator = DependencyValidator::new(temp.path());
    let violations = validator.validate_cargo_dependencies().unwrap();

    assert_eq!(violations.len(), 1);
    match &violations[0] {
        DependencyViolation::ForbiddenCargoDepedency {
            crate_name,
            forbidden_dep,
            ..
        } => {
            assert_eq!(crate_name, "mcb-domain");
            assert_eq!(forbidden_dep, "mcb-infrastructure");
        }
        _ => panic!("Expected ForbiddenCargoDependency"),
    }
}

#[rstest]
fn test_forbidden_use_statement() {
    let temp = TempDir::new().unwrap();

    let domain_dir = temp.path().join("crates/mcb-domain");
    fs::create_dir_all(domain_dir.join("src")).unwrap();
    fs::write(
        domain_dir.join("Cargo.toml"),
        r#"
[package]
name = "mcb-domain"
version = "0.1.1"
"#,
    )
    .unwrap();
    fs::write(
        domain_dir.join("src/lib.rs"),
        "use mcb_infrastructure::something;",
    )
    .unwrap();

    let validator = DependencyValidator::new(temp.path());
    let violations = validator.validate_use_statements().unwrap();

    assert_eq!(violations.len(), 1);
    match &violations[0] {
        DependencyViolation::ForbiddenUseStatement {
            crate_name,
            forbidden_dep,
            ..
        } => {
            assert_eq!(crate_name, "mcb-domain");
            assert_eq!(forbidden_dep, "mcb-infrastructure");
        }
        _ => panic!("Expected ForbiddenUseStatement"),
    }
}

#[rstest]
fn test_admin_bypass_boundary_blocks_non_allowlisted_imports() {
    let temp = TempDir::new().unwrap();
    let admin_web = temp.path().join("crates/mcb-server/src/admin/web");
    fs::create_dir_all(&admin_web).unwrap();
    fs::write(
        admin_web.join("entity_handlers.rs"),
        "use mcb_domain::ports::repositories::OrgEntityRepository;",
    )
    .unwrap();

    let admin_allowed = temp.path().join("crates/mcb-server/src/admin");
    fs::create_dir_all(&admin_allowed).unwrap();
    fs::write(
        admin_allowed.join("crud_adapter.rs"),
        "use mcb_domain::ports::repositories::OrgEntityRepository;",
    )
    .unwrap();

    let config_dir = temp.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("mcb-validate-internal.toml"),
        r#"
[[rules.dependency.bypass_boundaries]]
violation_id = "DEP004"
scan_root = "crates/mcb-server/src/admin"
pattern = "mcb_domain::ports::repositories"
allowed_files = [
    "crates/mcb-server/src/admin/crud_adapter.rs",
    "crates/mcb-server/src/admin/handlers.rs",
]
"#,
    )
    .unwrap();

    let validator = DependencyValidator::new(temp.path());
    let violations = validator.validate_bypass_boundaries().unwrap();

    assert_eq!(violations.len(), 1);
    assert!(matches!(
        &violations[0],
        DependencyViolation::AdminBypassImport { .. }
    ));
}

#[rstest]
fn test_cli_bypass_boundary_blocks_non_allowlisted_direct_validate_usage() {
    let temp = TempDir::new().unwrap();
    let cli_dir = temp.path().join("crates/mcb/src/cli");
    fs::create_dir_all(&cli_dir).unwrap();
    fs::write(
        cli_dir.join("lint.rs"),
        "use mcb_validate::ValidationConfig;",
    )
    .unwrap();
    fs::write(
        cli_dir.join("validate.rs"),
        "use mcb_validate::ValidationConfig;",
    )
    .unwrap();

    let config_dir = temp.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("mcb-validate-internal.toml"),
        r#"
[[rules.dependency.bypass_boundaries]]
violation_id = "DEP005"
scan_root = "crates/mcb/src/cli"
pattern = "mcb_validate::"
allowed_files = ["crates/mcb/src/cli/validate.rs"]
"#,
    )
    .unwrap();

    let validator = DependencyValidator::new(temp.path());
    let violations = validator.validate_bypass_boundaries().unwrap();

    assert_eq!(violations.len(), 1);
    assert!(matches!(
        &violations[0],
        DependencyViolation::CliBypassPath { .. }
    ));
}
