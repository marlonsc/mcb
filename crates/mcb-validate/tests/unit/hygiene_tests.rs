//! Tests for Test Hygiene Validation
//!
//! Validates `HygieneValidator` against fixture crates with precise
//! file + line + violation-type assertions.
//!
//! Note: `BadTestFileName` violations have no line field, so line=0
//! is used (skips line check).

use mcb_validate::HygieneValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_hygiene_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = HygieneValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── InlineTestModule ────────────────────────────────────────
            ("my-test/src/lib.rs", 366, "InlineTestModule"),
            // ── BadTestFileName — no line field, use 0 ──────────────────
            ("my-test/tests", 0, "BadTestFileName"),
            ("integration_test.rs", 0, "BadTestFileName"),
            ("integration_test.rs", 0, "BadTestFileName"),
            // ── BadTestFunctionName ─────────────────────────────────────
            ("integration_test.rs", 8, "BadTestFunctionName"),
            // ── TrivialAssertion ────────────────────────────────────────
            ("integration_test.rs", 4, "TrivialAssertion"),
            ("integration_test.rs", 10, "TrivialAssertion"),
            ("integration_test.rs", 16, "TrivialAssertion"),
        ],
        "HygieneValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_clean_hygiene_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        "
/// A well-structured module.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
",
    );
    let validator = HygieneValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(
        &violations,
        "Clean test organization should produce no violations",
    );
}
