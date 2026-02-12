//! Tests for Error Boundary Validation
//!
//! Validates `ErrorBoundaryValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use mcb_validate::ErrorBoundaryValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_error_boundary_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = ErrorBoundaryValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── MissingErrorContext (bare ?) ─────────────────────────────
            (SERVER_CRATE_HANDLER, 30, "MissingErrorContext"),
            (SERVER_CRATE_HANDLER, 32, "MissingErrorContext"),
            (SERVER_CRATE_HANDLER, 36, "MissingErrorContext"),
            (SERVER_CRATE_HANDLER, 106, "MissingErrorContext"),
            (SERVER_CRATE_HANDLER, 109, "MissingErrorContext"),
            (SERVER_CRATE_HANDLER, 112, "MissingErrorContext"),
            // ── WrongLayerError (infra types in domain) ─────────────────
            (DOMAIN_CRATE_SERVICE, 17, "WrongLayerError"),
            (DOMAIN_CRATE_SERVICE, 19, "WrongLayerError"),
            (DOMAIN_CRATE_SERVICE, 20, "WrongLayerError"),
            (DOMAIN_CRATE_SERVICE, 38, "WrongLayerError"),
            (DOMAIN_CRATE_SERVICE, 49, "WrongLayerError"),
            (DOMAIN_CRATE_SERVICE, 59, "WrongLayerError"),
            (DOMAIN_CRATE_SERVICE, 76, "WrongLayerError"),
            // ── LeakedInternalError (.to_string() in response) ──────────
            (SERVER_CRATE_HANDLER, 123, "LeakedInternalError"),
        ],
        "ErrorBoundaryValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_clean_error_boundary_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r##"
/// A function with proper error handling.
pub fn parse_config(input: &str) -> Result<i32, String> {
    input.parse::<i32>().map_err(|e| format!("parse error: {}", e))
}
"##,
    );
    let validator = ErrorBoundaryValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(
        &violations,
        "Clean error handling should produce no violations",
    );
}
