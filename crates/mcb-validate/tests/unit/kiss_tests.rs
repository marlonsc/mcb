//! Tests for KISS Validation
//!
//! Validates `KissValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use mcb_validate::KissValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_kiss_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = KissValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── FunctionTooManyParams (KISS002) ──────────────────────────
            (
                "my-domain/src/domain/service.rs",
                94,
                "FunctionTooManyParams",
            ),
            ("my-test/src/lib.rs", 77, "FunctionTooManyParams"),
            // ── BuilderTooComplex (KISS003) ──────────────────────────────
            ("my-test/src/lib.rs", 204, "BuilderTooComplex"),
            // ── DeepNesting (KISS004) ────────────────────────────────────
            ("my-test/src/lib.rs", 221, "DeepNesting"),
            // ── FunctionTooLong (KISS005) ────────────────────────────────
            ("my-test/src/lib.rs", 233, "FunctionTooLong"),
        ],
        "KissValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

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
    let validator = KissValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(&violations, "Clean code should produce no violations");
}
