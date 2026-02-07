//! Tests for Error Boundary Validation
//!
//! Uses centralized fixtures from `tests/fixtures/rust/` via shared test_utils
//! and constants from `test_constants`.

use mcb_validate::error_boundary::ErrorBoundaryValidator;
use tempfile::TempDir;

use crate::test_constants::{
    DOMAIN_CRATE, DOMAIN_ERROR_PATH, DOMAIN_SERVICE_PATH, FIXTURE_DOMAIN_ERROR_EXEMPT,
    FIXTURE_DOMAIN_WRONG_ERROR, FIXTURE_HANDLER_MISSING_CONTEXT, HANDLER_PATH, SERVER_CRATE,
};
use crate::test_utils::{assert_min_violations, assert_no_violations, setup_fixture_crate};

// ---------------------------------------------------------------------------
// validate_layer_error_types() — detects infrastructure error types in domain
// ---------------------------------------------------------------------------

#[test]
fn test_domain_infra_error_types_detected() {
    let temp = TempDir::new().unwrap();

    setup_fixture_crate(
        &temp,
        DOMAIN_CRATE,
        DOMAIN_SERVICE_PATH,
        FIXTURE_DOMAIN_WRONG_ERROR,
    );

    let validator = ErrorBoundaryValidator::new(temp.path());
    let violations = validator.validate_layer_error_types().unwrap();

    // The fixture has 6 distinct infra error types: std::io::Error, reqwest::Error,
    // sqlx::Error, serde_json::Error, tokio::*, hyper::Error
    assert_min_violations(
        &violations,
        3,
        "Should detect multiple infrastructure error types in domain",
    );

    // Verify different error types were found
    let error_types: Vec<&str> = violations
        .iter()
        .filter_map(|v| match v {
            mcb_validate::error_boundary::ErrorBoundaryViolation::WrongLayerError {
                error_type,
                ..
            } => Some(error_type.as_str()),
            _ => None,
        })
        .collect();

    assert!(
        error_types.iter().any(|t| t.contains("io")),
        "Should detect std::io::Error, found: {error_types:?}"
    );
}

#[test]
fn test_domain_error_module_exempt() {
    let temp = TempDir::new().unwrap();

    setup_fixture_crate(
        &temp,
        DOMAIN_CRATE,
        DOMAIN_ERROR_PATH,
        FIXTURE_DOMAIN_ERROR_EXEMPT,
    );

    let validator = ErrorBoundaryValidator::new(temp.path());
    let violations = validator.validate_layer_error_types().unwrap();

    assert_no_violations(
        &violations,
        "error.rs files should be exempt from layer error type checks",
    );
}

// ---------------------------------------------------------------------------
// validate_error_context() — detects bare ? without .context()/.map_err()
// ---------------------------------------------------------------------------

#[test]
fn test_handler_missing_error_context() {
    let temp = TempDir::new().unwrap();

    setup_fixture_crate(
        &temp,
        SERVER_CRATE,
        HANDLER_PATH,
        FIXTURE_HANDLER_MISSING_CONTEXT,
    );

    let validator = ErrorBoundaryValidator::new(temp.path());
    let violations = validator.validate_error_context().unwrap();

    assert_min_violations(
        &violations,
        2,
        "Should detect bare ? operators in handler code",
    );
}

// ---------------------------------------------------------------------------
// validate_leaked_errors() — detects internal error details in API responses
// ---------------------------------------------------------------------------

#[test]
fn test_handler_leaked_error_detection() {
    let temp = TempDir::new().unwrap();

    setup_fixture_crate(
        &temp,
        SERVER_CRATE,
        HANDLER_PATH,
        FIXTURE_HANDLER_MISSING_CONTEXT,
    );

    let validator = ErrorBoundaryValidator::new(temp.path());
    let violations = validator.validate_leaked_errors().unwrap();

    // The fixture has Debug formatting in responses and .to_string() leaks
    println!("Leaked error violations found: {}", violations.len());
    for v in &violations {
        println!("  - {v}");
    }
}

// ---------------------------------------------------------------------------
// validate_all() — combined validation
// ---------------------------------------------------------------------------

#[test]
fn test_domain_validate_all_with_violations() {
    let temp = TempDir::new().unwrap();

    setup_fixture_crate(
        &temp,
        DOMAIN_CRATE,
        DOMAIN_SERVICE_PATH,
        FIXTURE_DOMAIN_WRONG_ERROR,
    );

    let validator = ErrorBoundaryValidator::new(temp.path());
    let all_violations = validator.validate_all().unwrap();

    assert_min_violations(
        &all_violations,
        3,
        "validate_all should aggregate violations from domain fixture",
    );
}

#[test]
fn test_handler_validate_all_with_violations() {
    let temp = TempDir::new().unwrap();

    setup_fixture_crate(
        &temp,
        SERVER_CRATE,
        HANDLER_PATH,
        FIXTURE_HANDLER_MISSING_CONTEXT,
    );

    let validator = ErrorBoundaryValidator::new(temp.path());
    let all_violations = validator.validate_all().unwrap();

    assert_min_violations(
        &all_violations,
        2,
        "validate_all should aggregate violations from handler fixture",
    );
}
