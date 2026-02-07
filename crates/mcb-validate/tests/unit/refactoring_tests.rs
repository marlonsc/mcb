//! Tests for Refactoring Validation
//!
//! Discovery found 1 violation in the full workspace:
//! - Duplicate struct `User` defined in both `my-domain` and `my-server`

use mcb_validate::refactoring::RefactoringValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_refactoring_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = RefactoringValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violation_count(&violations, 1, "RefactoringValidator full workspace");
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-method tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_duplicate_definition_detection() {
    // Both my-domain and my-server define a `User` struct
    let (_temp, root) = with_fixture_workspace(&[DOMAIN_CRATE, SERVER_CRATE]);

    let validator = RefactoringValidator::new(&root);
    let violations = validator.validate_duplicate_definitions().unwrap();

    assert_min_violations(
        &violations,
        1,
        "duplicate User struct across domain and server crates",
    );
}

#[test]
fn test_no_duplicates_in_single_crate() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    let validator = RefactoringValidator::new(&root);
    let violations = validator.validate_duplicate_definitions().unwrap();

    // Single crate should have no cross-crate duplicate definitions
    assert_no_violations(
        &violations,
        "Single crate should have no cross-crate duplicates",
    );
}
