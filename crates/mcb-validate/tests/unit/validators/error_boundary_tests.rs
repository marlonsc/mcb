//! Tests for Error Boundary Validation
//!
//! Validates `ErrorBoundaryValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use crate::utils::test_constants::*;
use crate::utils::*;
use rstest::rstest;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_error_boundary_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let violations = run_named_validator(&root, "error_boundary").unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── MissingErrorContext (bare ?) ─────────────────────────────
            (
                "my-server/src/handlers/user_handler.rs",
                30,
                "MissingErrorContext",
            ),
            (
                "my-server/src/handlers/user_handler.rs",
                32,
                "MissingErrorContext",
            ),
            (
                "my-server/src/handlers/user_handler.rs",
                36,
                "MissingErrorContext",
            ),
            (
                "my-server/src/handlers/user_handler.rs",
                106,
                "MissingErrorContext",
            ),
            (
                "my-server/src/handlers/user_handler.rs",
                109,
                "MissingErrorContext",
            ),
            (
                "my-server/src/handlers/user_handler.rs",
                112,
                "MissingErrorContext",
            ),
            // ── WrongLayerError (infra types in domain) ─────────────────
            ("my-domain/src/domain/service.rs", 17, "WrongLayerError"),
            ("my-domain/src/domain/service.rs", 19, "WrongLayerError"),
            ("my-domain/src/domain/service.rs", 20, "WrongLayerError"),
            ("my-domain/src/domain/service.rs", 38, "WrongLayerError"),
            ("my-domain/src/domain/service.rs", 49, "WrongLayerError"),
            ("my-domain/src/domain/service.rs", 59, "WrongLayerError"),
            ("my-domain/src/domain/service.rs", 76, "WrongLayerError"),
            // ── LeakedInternalError (.to_string() in response) ──────────
            (
                "my-server/src/handlers/user_handler.rs",
                123,
                "LeakedInternalError",
            ),
        ],
        "ErrorBoundaryValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
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
    let violations = run_named_validator(&root, "error_boundary").unwrap();

    assert_no_violations(
        &violations,
        "Clean error handling should produce no violations",
    );
}
