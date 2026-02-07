//! Tests for test organization validation
//!
//! Discovery found 8 violations in the full workspace:
//! - InlineTestModule: #[cfg(test)] mod tests in my-test/src/lib.rs
//! - BadTestFunctionName: tests without test_ prefix
//! - Various test organization issues across fixture crates

use mcb_validate::tests_org::{TestValidator, TestViolation};

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_tests_org_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = TestValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violation_count(&violations, 8, "TestValidator full workspace");
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-method tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_inline_test_module_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has an inline #[cfg(test)] mod tests block
    let validator = TestValidator::new(&root);
    let violations = validator.validate_no_inline_tests().unwrap();

    assert_has_violation_matching(
        &violations,
        |v| matches!(v, TestViolation::InlineTestModule { .. }),
        "InlineTestModule in src/lib.rs",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_proper_test_file_no_violation() {
    // Create a crate with tests in the proper location only (no inline)
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
pub fn production_code() -> bool { true }
",
    );
    create_test_crate_with_tests(
        &_temp,
        TEST_CRATE,
        r"
pub fn production_code() -> bool { true }
",
        r"
#[test]
fn test_production_code() {
    assert!(my_test::production_code());
}
",
    );

    let validator = TestValidator::new(&root);
    let violations = validator.validate_no_inline_tests().unwrap();

    let inline_violations: Vec<_> = violations
        .iter()
        .filter(|v| matches!(v, TestViolation::InlineTestModule { .. }))
        .collect();

    assert_no_violations(
        &inline_violations,
        "Properly structured tests should not trigger inline test violation",
    );
}
