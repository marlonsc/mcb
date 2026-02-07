//! Tests for Code Organization Validation
//!
//! Discovery found 8 violations in the full workspace:
//! - Magic numbers in my-test (calculate_pricing) and my-domain
//! - File organization violations
//!
//! Uses fixture crates: `my-test`, `my-infra` (constants.rs exempt)

use mcb_validate::OrganizationValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_organization_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = OrganizationValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violation_count(&violations, 8, "OrganizationValidator full workspace");
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-method tests
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// Negative test
// ─────────────────────────────────────────────────────────────────────────────

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
