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
            (
                "my-server/src/handlers/user_handler.rs",
                42,
                "UnwrapInProduction",
            ),
            ("my-domain/src/domain/service.rs", 69, "UnwrapInProduction"),
            ("my-test/src/lib.rs", 17, "UnwrapInProduction"),
            ("my-test/src/lib.rs", 114, "UnwrapInProduction"),
            ("my-test/src/lib.rs", 294, "UnwrapInProduction"),
            // ── ExpectInProduction ──────────────────────────────────────
            (
                "my-server/src/handlers/user_handler.rs",
                91,
                "ExpectInProduction",
            ),
            ("my-test/src/lib.rs", 19, "ExpectInProduction"),
            // ── PanicInProduction ──────────────────────────────────────
            ("my-test/src/lib.rs", 39, "PanicInProduction"),
            // ── TodoComment ────────────────────────────────────────────
            ("my-domain/src/domain/service.rs", 66, "TodoComment"),
            ("my-domain/src/domain/service.rs", 71, "TodoComment"),
            ("my-domain/src/domain/service.rs", 151, "TodoComment"),
            ("my-domain/src/domain/service.rs", 159, "TodoComment"),
            ("my-domain/src/domain/service.rs", 180, "TodoComment"),
            ("my-domain/src/domain/service.rs", 183, "TodoComment"),
            ("my-test/src/lib.rs", 9, "TodoComment"),
            ("my-test/src/lib.rs", 15, "TodoComment"),
            ("my-test/src/lib.rs", 18, "TodoComment"),
            ("my-test/src/lib.rs", 24, "TodoComment"),
            ("my-test/src/lib.rs", 26, "TodoComment"),
            ("my-test/src/lib.rs", 334, "TodoComment"),
            ("my-test/src/lib.rs", 336, "TodoComment"),
            // ── DeadCodeAllowNotPermitted ──────────────────────────────
            ("my-test/src/lib.rs", 43, "DeadCodeAllowNotPermitted"),
            ("my-test/src/lib.rs", 44, "DeadCodeAllowNotPermitted"),
            ("my-test/src/lib.rs", 50, "DeadCodeAllowNotPermitted"),
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
