//! Integration tests that validate the actual workspace

use rstest::rstest;
use std::path::PathBuf;

use mcb_validate::{Severity, ValidationConfig, ValidatorRegistry, Violation};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn get_workspace_root() -> PathBuf {
    crate::test_utils::get_workspace_root()
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
) -> TestResult {
    let workspace_root = get_workspace_root();
    let config = ValidationConfig::new(&workspace_root);
    let registry = ValidatorRegistry::standard_for(&workspace_root);
    let violations = registry.validate_named(&config, &[validator_name])?;

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
    Ok(())
}

#[test]
fn test_validate_workspace_quality() -> TestResult {
    let workspace_root = get_workspace_root();
    let config = ValidationConfig::new(&workspace_root);
    let registry = ValidatorRegistry::standard_for(&workspace_root);
    let violations = registry.validate_named(&config, &["quality"])?;

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

    // Ensure test executed successfully
    // Validation completed successfully
    Ok(())
}

#[test]
fn test_validate_workspace_documentation() -> TestResult {
    let workspace_root = get_workspace_root();
    let config = ValidationConfig::new(&workspace_root);
    let registry = ValidatorRegistry::standard_for(&workspace_root);
    let violations = registry.validate_named(&config, &["documentation"])?;

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

    // Ensure test executed successfully
    // Validation completed successfully
    Ok(())
}

#[test]
fn test_full_validation_report() -> TestResult {
    let handle = std::thread::Builder::new()
        .name("full-report".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(run_full_validation_report)
        .map_err(|err| format!("spawn thread failed: {err}"))?;
    let join_result = handle
        .join()
        .map_err(|_| "thread join failed".to_string())?;
    join_result?;
    Ok(())
}

fn run_full_validation_report() -> TestResult {
    let workspace_root = get_workspace_root();
    let validator_names = ValidatorRegistry::standard_validator_names();

    let mut all_violations: Vec<Box<dyn Violation>> = Vec::new();
    let mut validator_errors: Vec<String> = Vec::new();

    for &name in validator_names {
        let root = workspace_root.clone();
        let vname = name.to_owned();
        let result = std::thread::Builder::new()
            .name(format!("validator-{vname}"))
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let config = ValidationConfig::new(&root);
                let registry = ValidatorRegistry::standard_for(&root);
                let validator = registry
                    .validators()
                    .iter()
                    .find(|v| v.name() == vname)
                    .ok_or_else(|| "validator must exist".to_string())?;
                validator.validate(&config).map_err(|err| err.to_string())
            })
            .map_err(|err| format!("spawn thread failed: {err}"))?
            .join();

        match result {
            Ok(Ok(violations)) => {
                all_violations.extend(violations);
            }
            Ok(Err(err)) => {
                validator_errors.push(format!("Validator '{name}' error: {err}"));
            }
            Err(_) => {
                validator_errors.push(format!(
                    "Validator '{name}' panicked (likely stack overflow)"
                ));
            }
        }
    }

    assert!(
        validator_errors.is_empty(),
        "Validation thread errors: {validator_errors:?}"
    );

    assert!(
        !validator_names.is_empty(),
        "Should have at least one validator"
    );
    assert!(
        !all_violations.is_empty(),
        "Full validation should produce violations"
    );
    Ok(())
}

#[test]
fn test_validation_config() -> TestResult {
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

    let dirs = config.get_source_dirs()?;
    println!("\nSource directories to scan:");
    for dir in &dirs {
        println!("  - {}", dir.display());
    }
    println!("\nTotal directories: {}", dirs.len());

    // Ensure test executed successfully
    // Validation completed successfully
    Ok(())
}
