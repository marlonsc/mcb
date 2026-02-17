//! Tests for Quality Validation
//!
//! Validates `QualityValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use mcb_validate::{QualityValidator, QualityViolation};
use rstest::rstest;

use crate::utils::test_constants::{
    DOMAIN_CRATE, FILE_SIZE_LOW_THRESHOLD, FIXTURE_DOMAIN_SERVICE_PATH,
    FIXTURE_SERVER_HANDLER_PATH, INFRA_CRATE, LIB_RS, SERVER_CRATE, TEST_CRATE,
};
use crate::utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_quality_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    let server_handler = format!("{SERVER_CRATE}/src/{FIXTURE_SERVER_HANDLER_PATH}");
    let domain_service = format!("{DOMAIN_CRATE}/src/{FIXTURE_DOMAIN_SERVICE_PATH}");
    let test_lib = format!("{TEST_CRATE}/src/{LIB_RS}");
    assert_violations_exact(
        &violations,
        &[
            // ── UnwrapInProduction ──────────────────────────────────────
            (&server_handler, 42, "UnwrapInProduction"),
            (&domain_service, 69, "UnwrapInProduction"),
            (&test_lib, 17, "UnwrapInProduction"),
            (&test_lib, 114, "UnwrapInProduction"),
            (&test_lib, 294, "UnwrapInProduction"),
            // ── ExpectInProduction ──────────────────────────────────────
            (&server_handler, 91, "ExpectInProduction"),
            (&test_lib, 19, "ExpectInProduction"),
            // ── PanicInProduction ──────────────────────────────────────
            (&test_lib, 39, "PanicInProduction"),
            // ── TodoComment ────────────────────────────────────────────
            (&domain_service, 66, "TodoComment"),
            (&domain_service, 71, "TodoComment"),
            (&domain_service, 151, "TodoComment"),
            (&domain_service, 159, "TodoComment"),
            (&domain_service, 180, "TodoComment"),
            (&domain_service, 183, "TodoComment"),
            (&test_lib, 9, "TodoComment"),
            (&test_lib, 15, "TodoComment"),
            (&test_lib, 18, "TodoComment"),
            (&test_lib, 24, "TodoComment"),
            (&test_lib, 26, "TodoComment"),
            (&test_lib, 334, "TodoComment"),
            (&test_lib, 336, "TodoComment"),
            // ── DeadCodeAllowNotPermitted ──────────────────────────────
            (&test_lib, 43, "DeadCodeAllowNotPermitted"),
            (&test_lib, 44, "DeadCodeAllowNotPermitted"),
            (&test_lib, 50, "DeadCodeAllowNotPermitted"),
        ],
        "QualityValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// validate_file_sizes() — threshold-based
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[case(Some(FILE_SIZE_LOW_THRESHOLD), true)]
#[case(None, false)]
fn file_size_threshold_behavior(
    #[case] max_lines: Option<usize>,
    #[case] expect_file_too_large: bool,
) {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = if let Some(max) = max_lines {
        QualityValidator::new(&root).with_max_file_lines(max)
    } else {
        QualityValidator::new(&root)
    };
    let violations = validator.validate_all().unwrap();

    if expect_file_too_large {
        assert_has_violation_matching(
            &violations,
            |v| matches!(v, QualityViolation::FileTooLarge { .. }),
            "FileTooLarge",
        );
    } else {
        // Default threshold (500 lines) should NOT trigger FileTooLarge,
        // but other quality violations (hygiene, todo, etc.) may still appear.
        let file_too_large_violations: Vec<_> = violations
            .iter()
            .filter(|v| matches!(v, QualityViolation::FileTooLarge { .. }))
            .collect();
        assert!(
            file_too_large_violations.is_empty(),
            "Fixture file under default threshold (500): expected no FileTooLarge violations, got {} - {:?}",
            file_too_large_violations.len(),
            file_too_large_violations
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Exemptions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_unwrap_exempt_in_test_code() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

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
        "
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
