//! Tests for Implementation Quality Validation
//!
//! Validates `ImplementationQualityValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use mcb_validate::ImplementationQualityValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_implementation_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = ImplementationQualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── StubMacro ───────────────────────────────────────────────
            (DOMAIN_CRATE_SERVICE, 159, "StubMacro"),
            (DOMAIN_CRATE_SERVICE, 165, "StubMacro"),
            (DOMAIN_CRATE_SERVICE, 180, "StubMacro"),
            (DOMAIN_CRATE_SERVICE, 183, "StubMacro"),
            (TEST_CRATE_LIB, 336, "StubMacro"),
            // ── EmptyCatchAll ───────────────────────────────────────────
            (DOMAIN_CRATE_SERVICE, 130, "EmptyCatchAll"),
            (TEST_CRATE_LIB, 357, "EmptyCatchAll"),
        ],
        "ImplementationQualityValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_clean_implementation_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
/// A well-implemented function.
pub fn compute(x: i32) -> i32 {
    x * 2 + 1
}
",
    );
    let validator = ImplementationQualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(
        &violations,
        "Clean implementation should produce no violations",
    );
}
