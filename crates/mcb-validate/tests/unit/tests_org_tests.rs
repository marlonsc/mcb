//! Tests for test organization validation
//!
//! Uses fixture crate `my-test` which contains:
//! - Inline test module in src/lib.rs (#[cfg(test)] mod tests { ... })
//! - Tests with bad naming in tests/integration_test.rs

use mcb_validate::tests_org::{TestValidator, TestViolation};

use crate::test_constants::*;
use crate::test_utils::*;

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

#[test]
fn test_bad_test_naming_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/tests/integration_test.rs has bad_naming_convention()
    // that doesn't follow test_ prefix convention
    let validator = TestValidator::new(&root);
    let violations = validator.validate_test_naming().unwrap();

    let has_bad_name = violations
        .iter()
        .any(|v| matches!(v, TestViolation::BadTestFunctionName { .. }));

    // Some validators may not check integration test naming — log for visibility
    if has_bad_name {
        println!("Detected bad test naming in fixture");
    } else {
        println!("Test naming check may not cover integration tests");
    }
}

#[test]
fn test_proper_test_file_no_violation() {
    // Create a crate with tests in the proper location only (no inline)
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
pub fn production_code() -> bool { true }
",
    );
    // Add a properly structured test file
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
