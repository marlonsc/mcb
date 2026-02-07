//! Tests for Code Organization Validation
//!
//! Uses fixture crates:
//! - `my-test`: contains magic numbers in calculate_pricing()
//! - `my-infra`: contains constants.rs (should be exempt from magic number checks)

use mcb_validate::OrganizationValidator;

use crate::test_constants::*;
use crate::test_utils::*;

#[test]
fn test_magic_number_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has calculate_pricing() with magic numbers:
    //   0.0875 (tax rate), 15.99 (shipping), 10000.0 (threshold), 0.95 (discount)
    let validator = OrganizationValidator::new(&root);
    let violations = validator.validate_magic_numbers().unwrap();

    assert_min_violations(&violations, 1, "magic numbers in calculate_pricing()");
}

#[test]
fn test_constants_file_exempt() {
    let (_temp, root) = with_fixture_crate(INFRA_CRATE);

    // my-infra/src/constants.rs has numeric constants like MAX_DB_CONNECTIONS = 100
    // These should NOT trigger magic number violations
    let validator = OrganizationValidator::new(&root);
    let violations = validator.validate_magic_numbers().unwrap();

    assert_no_violation_from_file(&violations, "constants.rs");
}

#[test]
fn test_no_magic_numbers_in_clean_code() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
pub fn clean_function() {
    let x = 0;
    let y = 1;
}
",
    );

    let validator = OrganizationValidator::new(&root);
    let violations = validator.validate_magic_numbers().unwrap();

    assert_no_violations(
        &violations,
        "Small literals (0, 1) should not trigger magic number detection",
    );
}
