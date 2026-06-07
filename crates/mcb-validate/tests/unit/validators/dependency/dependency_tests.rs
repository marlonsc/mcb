use std::fs;

use mcb_domain::utils::tests::utils::TestResult;
use mcb_validate::{DependencyValidator, DependencyViolation};
use rstest::*;
use tempfile::TempDir;

fn create_test_workspace() -> Result<TempDir, Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;

    fs::write(
        temp.path().join("Cargo.toml"),
        r#"
[workspace]
members = ["crates/mcb-domain", "crates/mcb-infrastructure"]
"#,
    )?;

    let domain_dir = temp.path().join("crates/mcb-domain");
    fs::create_dir_all(domain_dir.join("src"))?;
    fs::write(
        domain_dir.join("Cargo.toml"),
        r#"
[package]
name = "mcb-domain"
version = "0.1.1"

[dependencies]
serde = "1.0"
"#,
    )?;
    fs::write(domain_dir.join("src/lib.rs"), "pub fn domain() {}")?;

    let infra_dir = temp.path().join("crates/mcb-infrastructure");
    fs::create_dir_all(infra_dir.join("src"))?;
    fs::write(
        infra_dir.join("Cargo.toml"),
        r#"
[package]
name = "mcb-infrastructure"
version = "0.1.1"

[dependencies]
mcb-domain = { path = "../mcb-domain" }
"#,
    )?;
    fs::write(
        infra_dir.join("src/lib.rs"),
        "use mcb_domain::domain;\npub fn infra() { domain(); }",
    )?;

    Ok(temp)
}

#[rstest]
fn test_valid_dependencies() -> TestResult {
    let temp = create_test_workspace()?;
    let validator = DependencyValidator::new(temp.path());

    let violations = validator.validate_cargo_dependencies()?;
    assert!(
        violations.is_empty(),
        "Expected no violations, got: {violations:?}"
    );
    Ok(())
}

#[rstest]
fn test_forbidden_dependency() -> TestResult {
    let temp = TempDir::new()?;

    let domain_dir = temp.path().join("crates/mcb-domain");
    fs::create_dir_all(domain_dir.join("src"))?;
    fs::write(
        domain_dir.join("Cargo.toml"),
        r#"
[package]
name = "mcb-domain"
version = "0.1.1"

[dependencies]
mcb-infrastructure = { path = "../mcb-infrastructure" }
"#,
    )?;
    fs::write(domain_dir.join("src/lib.rs"), "")?;

    let validator = DependencyValidator::new(temp.path());
    let violations = validator.validate_cargo_dependencies()?;

    assert_eq!(violations.len(), 1);
    let violation = &violations[0];
    match violation {
        DependencyViolation::ForbiddenCargoDepedency {
            crate_name,
            forbidden_dep,
            ..
        } => {
            assert_eq!(crate_name, "mcb-domain");
            assert_eq!(forbidden_dep, "mcb-infrastructure");
        }
        DependencyViolation::ForbiddenUseStatement { .. }
        | DependencyViolation::CircularDependency { .. }
        | DependencyViolation::AdminBypassImport { .. }
        | DependencyViolation::CliBypassPath { .. } => {
            return Err(format!("Expected ForbiddenCargoDependency, got {violation:?}").into());
        }
    }
    Ok(())
}

#[rstest]
fn test_forbidden_use_statement() -> TestResult {
    let temp = TempDir::new()?;

    let domain_dir = temp.path().join("crates/mcb-domain");
    fs::create_dir_all(domain_dir.join("src"))?;
    fs::write(
        domain_dir.join("Cargo.toml"),
        r#"
[package]
name = "mcb-domain"
version = "0.1.1"
"#,
    )?;
    fs::write(
        domain_dir.join("src/lib.rs"),
        "use mcb_infrastructure::something;",
    )?;

    let validator = DependencyValidator::new(temp.path());
    let violations = validator.validate_use_statements()?;

    assert_eq!(violations.len(), 1);
    let violation = &violations[0];
    match violation {
        DependencyViolation::ForbiddenUseStatement {
            crate_name,
            forbidden_dep,
            ..
        } => {
            assert_eq!(crate_name, "mcb-domain");
            assert_eq!(forbidden_dep, "mcb-infrastructure");
        }
        DependencyViolation::ForbiddenCargoDepedency { .. }
        | DependencyViolation::CircularDependency { .. }
        | DependencyViolation::AdminBypassImport { .. }
        | DependencyViolation::CliBypassPath { .. } => {
            return Err(format!("Expected ForbiddenUseStatement, got {violation:?}").into());
        }
    }
    Ok(())
}

#[rstest]
fn test_admin_bypass_boundary_blocks_non_allowlisted_imports() -> TestResult {
    let temp = TempDir::new()?;
    let admin_web = temp.path().join("crates/mcb-server/src/admin/web");
    fs::create_dir_all(&admin_web)?;
    fs::write(
        admin_web.join("entity_handlers.rs"),
        "use mcb_domain::ports::OrgEntityRepository;",
    )?;

    let admin_allowed = temp.path().join("crates/mcb-server/src/admin");
    fs::create_dir_all(&admin_allowed)?;
    fs::write(
        admin_allowed.join("crud_adapter.rs"),
        "use mcb_domain::ports::OrgEntityRepository;",
    )?;

    let config_dir = temp.path().join("config");
    fs::create_dir_all(&config_dir)?;
    fs::write(
        config_dir.join("mcb-validate-internal.toml"),
        r#"
[[rules.dependency.bypass_boundaries]]
violation_id = "DEP004"
scan_root = "crates/mcb-server/src/admin"
 pattern = "mcb_domain::ports::OrgEntityRepository"
allowed_files = [
    "crates/mcb-server/src/admin/crud_adapter.rs",
    "crates/mcb-server/src/admin/handlers.rs",
]
"#,
    )?;

    let validator = DependencyValidator::new(temp.path());
    let violations = validator.validate_bypass_boundaries()?;

    assert_eq!(violations.len(), 1);
    assert!(matches!(
        &violations[0],
        DependencyViolation::AdminBypassImport { .. }
    ));
    Ok(())
}

#[rstest]
fn test_cli_bypass_boundary_blocks_non_allowlisted_direct_validate_usage() -> TestResult {
    let temp = TempDir::new()?;
    let cli_dir = temp.path().join("crates/mcb/src/cli");
    fs::create_dir_all(&cli_dir)?;
    fs::write(
        cli_dir.join("lint.rs"),
        "use mcb_validate::ValidationConfig;",
    )?;
    fs::write(
        cli_dir.join("validate.rs"),
        "use mcb_validate::ValidationConfig;",
    )?;

    let config_dir = temp.path().join("config");
    fs::create_dir_all(&config_dir)?;
    fs::write(
        config_dir.join("mcb-validate-internal.toml"),
        r#"
[[rules.dependency.bypass_boundaries]]
violation_id = "DEP005"
scan_root = "crates/mcb/src/cli"
pattern = "mcb_validate::"
allowed_files = ["crates/mcb/src/cli/validate.rs"]
"#,
    )?;

    let validator = DependencyValidator::new(temp.path());
    let violations = validator.validate_bypass_boundaries()?;

    assert_eq!(violations.len(), 1);
    assert!(matches!(
        &violations[0],
        DependencyViolation::CliBypassPath { .. }
    ));
    Ok(())
}
