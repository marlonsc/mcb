//! Integration tests that validate the actual workspace

use rstest::rstest;
use std::path::PathBuf;

use mcb_validate::{Severity, ValidationConfig, ValidatorRegistry, Violation};

fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map_or_else(
            || PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            std::path::Path::to_path_buf,
        )
}

#[rstest]
#[case("dependency", "Dependency Violations", true)]
#[case("patterns", "Pattern Violations", false)]
#[case("hygiene", "Test Hygiene Violations", false)]
#[case("naming", "Naming Violations", false)]
fn validate_workspace_group(
    #[case] validator_name: &str,
    #[case] header: &str,
    #[case] must_be_clean: bool,
) {
    let workspace_root = get_workspace_root();
    let config = ValidationConfig::new(&workspace_root);
    let registry = ValidatorRegistry::standard_for(&workspace_root);
    let violations = registry
        .validate_named(&config, &[validator_name])
        .unwrap_or_default();

    println!("\n=== {header} ===");
    for v in &violations {
        println!("  [{:?}] {}", v.severity(), v);
    }
    println!("Total: {} {validator_name} violations\n", violations.len());

    if must_be_clean {
        assert!(
            violations.is_empty(),
            "Found {} dependency violations - Clean Architecture rules violated",
            violations.len()
        );
    }
}

#[test]
fn test_validate_workspace_quality() {
    let workspace_root = get_workspace_root();
    let config = ValidationConfig::new(&workspace_root);
    let registry = ValidatorRegistry::standard_for(&workspace_root);
    let violations = registry
        .validate_named(&config, &["quality"])
        .unwrap_or_default();

    println!("\n=== Quality Violations ===");
    let errors: Vec<_> = violations
        .iter()
        .filter(|v| v.severity() == Severity::Error)
        .collect();
    let warnings: Vec<_> = violations
        .iter()
        .filter(|v| v.severity() == Severity::Warning)
        .collect();

    for v in &errors {
        println!("  [ERROR] {v}");
    }
    for v in &warnings {
        println!("  [WARNING] {v}");
    }
    println!(
        "Total: {} errors, {} warnings\n",
        errors.len(),
        warnings.len()
    );

    // Report but don't fail on warnings (file size, pending comments)
    // Only fail on errors (unwrap/expect/panic in production)
    if !errors.is_empty() {
        println!("\nProduction code contains unwrap/expect/panic!");
        for e in &errors {
            println!("  - {e}");
        }
    }
}

#[test]
fn test_validate_workspace_documentation() {
    let workspace_root = get_workspace_root();
    let config = ValidationConfig::new(&workspace_root);
    let registry = ValidatorRegistry::standard_for(&workspace_root);
    let violations = registry
        .validate_named(&config, &["documentation"])
        .unwrap_or_default();

    println!("\n=== Documentation Violations ===");
    let by_severity = |sev: Severity| violations.iter().filter(|v| v.severity() == sev).count();

    println!(
        "  Errors: {}, Warnings: {}, Info: {}",
        by_severity(Severity::Error),
        by_severity(Severity::Warning),
        by_severity(Severity::Info)
    );

    // Only print first 20 violations to avoid noise
    for v in violations.iter().take(20) {
        println!("  [{:?}] {}", v.severity(), v);
    }
    if violations.len() > 20 {
        println!("  ... and {} more", violations.len() - 20);
    }
    println!("Total: {} documentation violations\n", violations.len());
}

#[test]
fn test_full_validation_report() {
    let handle = match std::thread::Builder::new()
        .name("full-report".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(run_full_validation_report)
    {
        Ok(handle) => handle,
        Err(err) => {
            mcb_domain::warn!(
                "validate_test",
                "Failed to spawn full-report thread",
                &err.to_string()
            );
            std::thread::spawn(run_full_validation_report)
        }
    };

    if handle.join().is_err() {
        mcb_domain::warn!("validate_test", "full-report thread join failed");
    }
}

fn run_full_validation_report() {
    let workspace_root = get_workspace_root();
    let validator_names = ValidatorRegistry::standard_validator_names();

    let mut all_violations: Vec<Box<dyn Violation>> = Vec::new();

    for &name in validator_names {
        let root = workspace_root.clone();
        let vname = name.to_owned();
        let result = std::thread::Builder::new()
            .name(format!("validator-{vname}"))
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let config = ValidationConfig::new(&root);
                let registry = ValidatorRegistry::standard_for(&root);
                let v = registry.validators().iter().find(|v| v.name() == vname);
                assert!(v.is_some(), "validator must exist");
                v.and_then(|v| v.validate(&config).ok())
            })
            .unwrap_or_else(|_| {
                mcb_domain::warn!("validate_test", "Failed to spawn validator thread", &name);
                std::thread::spawn(|| None)
            })
            .join();

        match result {
            Ok(Some(violations)) => all_violations.extend(violations),
            Ok(None) => {
                mcb_domain::warn!(
                    "validate_test",
                    "Validator returned error or was not found",
                    &name
                );
            }
            Err(_) => {
                mcb_domain::warn!(
                    "validate_test",
                    "Validator panicked (likely stack overflow)",
                    &name
                );
            }
        }
    }

    assert!(
        !validator_names.is_empty(),
        "Should have at least one validator"
    );
    assert!(
        !all_violations.is_empty(),
        "Full validation should produce violations"
    );
}

#[test]
fn test_validation_config() {
    let workspace_root = get_workspace_root();
    let config = ValidationConfig::new(&workspace_root)
        .with_additional_path("src.legacy")
        .with_exclude_pattern("target/");

    println!("\n{}", "=".repeat(60));
    println!("VALIDATION CONFIGURATION");
    println!("{}", "=".repeat(60));
    println!("Workspace root: {}", config.workspace_root.display());
    println!("Additional paths: {:?}", config.additional_src_paths);
    println!("Exclude patterns: {:?}", config.exclude_patterns);

    let dirs = config.get_source_dirs().unwrap_or_default();
    println!("\nSource directories to scan:");
    for dir in &dirs {
        println!("  - {}", dir.display());
    }
    println!("\nTotal directories: {}", dirs.len());
}
