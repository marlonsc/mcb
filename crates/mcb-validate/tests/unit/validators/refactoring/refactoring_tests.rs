//! Tests for Refactoring Validation
//!
//! Validates `RefactoringValidator` against fixture crates with precise
//! file + line + violation-type assertions.
//!
//! Note: `DuplicateDefinition` has no single line field; it references
//! multiple files. We use line=0 to skip line check.

use mcb_validate::RefactoringValidator;

use crate::utils::test_constants::*;
use crate::utils::*;
use rstest::rstest;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_refactoring_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = RefactoringValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    // 1 violation: DuplicateDefinition for 'User' across my-server and my-domain
    assert_violations_exact(
        &violations,
        &[("User", 0, "DuplicateDefinition")],
        "RefactoringValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_clean_refactoring_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        "
/// A unique type with no duplicates.
pub struct UniqueType {
    /// Value field.
    pub value: i32,
}
",
    );
    let validator = RefactoringValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(
        &violations,
        "Clean code with unique types should produce no violations",
    );
}
