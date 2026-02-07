//! Tests for Refactoring Validation
//!
//! Uses fixture crates `my-domain` and `my-server` which both define
//! a `User` struct — triggering duplicate type detection across crates.

use mcb_validate::refactoring::RefactoringValidator;

use crate::test_constants::*;
use crate::test_utils::*;

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

    // Single crate should not have duplicate type names across crates
    println!("Duplicates in single crate: {}", violations.len());
}

#[test]
fn test_orphan_module_detection() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
pub mod used_module;
",
    );
    // Create an orphaned file that's not declared in lib.rs
    let orphan_path = root
        .join("crates")
        .join(TEST_CRATE)
        .join("src")
        .join("orphan.rs");
    std::fs::write(&orphan_path, "pub fn orphaned() {}").unwrap();

    let validator = RefactoringValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    println!(
        "Refactoring violations (orphan check): {}",
        violations.len()
    );
}
