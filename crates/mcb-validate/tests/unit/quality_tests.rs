//! Tests for Quality Validation
//!
//! Validates `QualityValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use mcb_validate::{QualityValidator, QualityViolation};

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_quality_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── UnwrapInProduction ──────────────────────────────────────
            (SERVER_CRATE_HANDLER, 42, "UnwrapInProduction"),
            (DOMAIN_CRATE_SERVICE, 69, "UnwrapInProduction"),
            (TEST_CRATE_LIB, 17, "UnwrapInProduction"),
            (TEST_CRATE_LIB, 114, "UnwrapInProduction"),
            (TEST_CRATE_LIB, 294, "UnwrapInProduction"),
            // ── ExpectInProduction ──────────────────────────────────────
            (SERVER_CRATE_HANDLER, 91, "ExpectInProduction"),
            (TEST_CRATE_LIB, 19, "ExpectInProduction"),
            // ── PanicInProduction ──────────────────────────────────────
            (TEST_CRATE_LIB, 39, "PanicInProduction"),
            // ── TodoComment ────────────────────────────────────────────
            (DOMAIN_CRATE_SERVICE, 66, "TodoComment"),
            (DOMAIN_CRATE_SERVICE, 71, "TodoComment"),
            (DOMAIN_CRATE_SERVICE, 151, "TodoComment"),
            (DOMAIN_CRATE_SERVICE, 159, "TodoComment"),
            (DOMAIN_CRATE_SERVICE, 180, "TodoComment"),
            (DOMAIN_CRATE_SERVICE, 183, "TodoComment"),
            (TEST_CRATE_LIB, 9, "TodoComment"),
            (TEST_CRATE_LIB, 15, "TodoComment"),
            (TEST_CRATE_LIB, 18, "TodoComment"),
            (TEST_CRATE_LIB, 24, "TodoComment"),
            (TEST_CRATE_LIB, 26, "TodoComment"),
            (TEST_CRATE_LIB, 334, "TodoComment"),
            (TEST_CRATE_LIB, 336, "TodoComment"),
            // ── DeadCodeAllowNotPermitted ──────────────────────────────
            (TEST_CRATE_LIB, 43, "DeadCodeAllowNotPermitted"),
            (TEST_CRATE_LIB, 44, "DeadCodeAllowNotPermitted"),
            (TEST_CRATE_LIB, 50, "DeadCodeAllowNotPermitted"),
        ],
        "QualityValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// validate_file_sizes() — threshold-based
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_file_size_with_low_threshold() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = QualityValidator::new(&root).with_max_file_lines(FILE_SIZE_LOW_THRESHOLD);
    let violations = validator.validate_file_sizes().unwrap();

    assert_has_violation_matching(
        &violations,
        |v| matches!(v, QualityViolation::FileTooLarge { .. }),
        "FileTooLarge",
    );
}

#[test]
fn test_file_size_default_no_violation() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_file_sizes().unwrap();

    assert_no_violations(&violations, "Fixture file under default threshold (500)");
}

// ─────────────────────────────────────────────────────────────────────────────
// Exemptions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_unwrap_exempt_in_test_code() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_no_unwrap_expect().unwrap();

    // #[cfg(test)] mod tests in lib.rs should be completely exempt.
    assert_no_violation_from_file(&violations, "mod tests");
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_clean_code_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
/// A well-documented function.
pub fn safe_parse(input: &str) -> Result<i32, std::num::ParseIntError> {
    input.parse()
}
",
    );
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(&violations, "Clean code should produce no violations");
}
