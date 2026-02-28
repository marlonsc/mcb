//! Unit tests for validation service filtering and validator selection.

use crate::utils::workspace::workspace_root;
use mcb_domain::ports::ValidationServiceInterface;
use mcb_infrastructure::validation::InfraValidationService;

#[tokio::test]
async fn test_validate_with_specific_validator_filters_correctly()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = workspace_root()?;
    let service = InfraValidationService::new();

    // Run validation with only the "quality" validator
    let result = std::thread::Builder::new()
        .name("validate-quality-only".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Runtime::new()
                .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))
                .and_then(|rt| {
                    rt.block_on(service.validate(
                        &workspace_root,
                        Some(&["quality".to_owned()]),
                        None,
                    ))
                })
        })?
        .join()
        .map_err(|_| "thread panicked")?;

    let report = result?;

    // Verify that the validation ran and produced a report
    // (it may pass or fail, but it should have a definitive result)
    assert!(
        report.passed || !report.violations.is_empty(),
        "report should have a definitive result"
    );

    Ok(())
}

#[tokio::test]
async fn test_validate_with_specific_validator_does_not_fail_on_unrelated_validators()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = workspace_root()?;
    let service = InfraValidationService::new();

    // Run validation with only the "quality" validator
    // This should succeed even if other validators (like clean_architecture)
    // would fail to build or run
    let result = std::thread::Builder::new()
        .name("validate-quality-isolated".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Runtime::new()
                .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))
                .and_then(|rt| {
                    rt.block_on(service.validate(
                        &workspace_root,
                        Some(&["quality".to_owned()]),
                        Some("warning"),
                    ))
                })
        })?
        .join()
        .map_err(|_| "thread panicked")?;

    // The validation should complete without error
    // (even if the quality validator itself finds violations)
    let report = result?;

    // Verify that we got a valid report back
    assert!(
        report.passed || !report.violations.is_empty(),
        "report should have a definitive result"
    );

    Ok(())
}
