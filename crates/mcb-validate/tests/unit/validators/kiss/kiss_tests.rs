//! Tests for KISS Validation
//!
//! Validates `KissValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use mcb_domain::utils::test_constants::{
    DOMAIN_CRATE, FIXTURE_DOMAIN_SERVICE_PATH, INFRA_CRATE, LIB_RS, SERVER_CRATE, TEST_CRATE,
};
use mcb_domain::utils::*;
use rstest::rstest;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_kiss_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let violations = run_named_validator(&root, "kiss").unwrap();

    let domain_service = format!("{DOMAIN_CRATE}/src/{FIXTURE_DOMAIN_SERVICE_PATH}");
    let test_lib = format!("{TEST_CRATE}/src/{LIB_RS}");
    assert_violations_exact(
        &violations,
        &[
            // ── FunctionTooManyParams (KISS002) ──────────────────────────
            (&domain_service, 94, "FunctionTooManyParams"),
            (&test_lib, 77, "FunctionTooManyParams"),
            // ── BuilderTooComplex (KISS003) ──────────────────────────────
            (&test_lib, 204, "BuilderTooComplex"),
            // ── DeepNesting (KISS004) ────────────────────────────────────
            (&test_lib, 221, "DeepNesting"),
            // ── FunctionTooLong (KISS005) ────────────────────────────────
            (&test_lib, 233, "FunctionTooLong"),
        ],
        "KissValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_clean_code_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r#"
/// A simple function with few parameters.
pub fn greet(name: &str, greeting: &str) -> String {
    format!("{} {}", greeting, name)
}
"#,
    );
    let violations = run_named_validator(&root, "kiss").unwrap();

    assert_no_violations(&violations, "Clean code should produce no violations");
}
