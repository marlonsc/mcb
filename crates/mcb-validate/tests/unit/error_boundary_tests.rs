//! Tests for Error Boundary Validation
//!
//! Discovery found 22 violations in the full workspace:
//! - Wrong-layer error types in domain code (std::io::Error, reqwest, sqlx, etc.)
//! - Missing error context (bare `?` operators)
//! - Leaked error details in API responses
//!
//! Uses fixture crates via `with_fixture_workspace` and `with_fixture_crate`.

use mcb_validate::error_boundary::{ErrorBoundaryValidator, ErrorBoundaryViolation};

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_error_boundary_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = ErrorBoundaryValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violation_count(&violations, 22, "ErrorBoundaryValidator full workspace");
}

// ─────────────────────────────────────────────────────────────────────────────
// validate_layer_error_types() — wrong layer errors in domain
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_domain_infra_error_types_detected() {
    let (_temp, root) = with_fixture_crate(DOMAIN_CRATE);

    let validator = ErrorBoundaryValidator::new(&root);
    let violations = validator.validate_layer_error_types().unwrap();

    // my-domain has infra error types: std::io::Error, reqwest::Error,
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
            ErrorBoundaryViolation::WrongLayerError { error_type, .. } => Some(error_type.as_str()),
            _ => None,
        })
        .collect();

    assert!(
        error_types.iter().any(|t| t.contains("io")),
        "Should detect std::io::Error, found: {error_types:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// validate_error_context() — bare ? operators
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_handler_missing_error_context() {
    let (_temp, root) = with_fixture_crate(SERVER_CRATE);

    let validator = ErrorBoundaryValidator::new(&root);
    let violations = validator.validate_error_context().unwrap();

    assert_min_violations(
        &violations,
        1,
        "Should detect bare ? operators in handler code",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: error.rs files should be exempt
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_domain_error_module_exempt() {
    let (_temp, root) = with_fixture_crate(DOMAIN_CRATE);

    let validator = ErrorBoundaryValidator::new(&root);
    let violations = validator.validate_layer_error_types().unwrap();

    // error.rs files are exempt from layer error type checks
    assert_no_violation_from_file(&violations, "error.rs");
}
