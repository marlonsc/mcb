//! Tests for Quality Validation
//!
//! Validates `QualityValidator` against fixture crates. Discovery found 23
//! violations across the full workspace (unwrap/expect, TODO/FIXME, panic,
//! dead_code annotations).
//!
//! | Method                             | Fixture violations                                     |
//! |------------------------------------|--------------------------------------------------------|
//! | `validate_no_unwrap_expect()`      | 3 — `.unwrap()` (×2), `.expect()` (×1) in `my-test`   |
//! | `find_todo_comments()`             | 14 — TODO/FIXME across `my-test` and `my-domain`       |
//! | `validate_no_panic()`              | 1 — `panic!()` in `validate_critical_config()`         |
//! | `validate_dead_code_annotations()` | 3 — `#[allow(dead_code)]` on struct + 2 fields         |
//! | `validate_file_sizes()`            | 0 with default threshold, 1+ with low threshold        |
//! | `validate_all()`                   | 23 total (with `.expect()` as separate violation)       |

use mcb_validate::{QualityValidator, QualityViolation};

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_quality_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violation_count(&violations, 23, "QualityValidator full workspace");
}

// ─────────────────────────────────────────────────────────────────────────────
// validate_no_unwrap_expect()
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_unwrap_expect_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_no_unwrap_expect().unwrap();

    // my-test/src/lib.rs:
    //   L17: std::fs::read_to_string(path).unwrap()
    //   L19: serde_json::from_str(&content).expect("invalid config JSON")
    //   L114: data.lock().unwrap()
    assert_min_violations(&violations, 2, "unwrap/expect in fixture");
}

#[test]
fn test_unwrap_exempt_in_test_code() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_no_unwrap_expect().unwrap();

    // #[cfg(test)] mod tests in lib.rs should be completely exempt.
    assert_no_violation_from_file(&violations, "mod tests");
}

// ─────────────────────────────────────────────────────────────────────────────
// find_todo_comments()
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_todo_fixme_detection() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = QualityValidator::new(&root);
    let violations = validator.find_todo_comments().unwrap();

    // Discovery found 14 TODO/FIXME across my-test and my-domain
    assert_violation_count(&violations, 14, "TODO and FIXME comments across workspace");
}

// ─────────────────────────────────────────────────────────────────────────────
// validate_no_panic()
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_panic_in_production_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_no_panic().unwrap();

    // validate_critical_config(): panic!("Configuration must be a JSON object")
    assert_min_violations(&violations, 1, "panic!() in fixture");
}

// ─────────────────────────────────────────────────────────────────────────────
// validate_dead_code_annotations()
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_dead_code_annotation_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_dead_code_annotations().unwrap();

    // #[allow(dead_code)] on InternalCache struct fields + unused_helper() fn
    assert_min_violations(&violations, 1, "#[allow(dead_code)]");
}

// ─────────────────────────────────────────────────────────────────────────────
// validate_file_sizes()
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_file_size_with_low_threshold() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = QualityValidator::new(&root).with_max_file_lines(FILE_SIZE_LOW_THRESHOLD);
    let violations = validator.validate_file_sizes().unwrap();

    // Fixture lib.rs has ~250 lines — exceeds FILE_SIZE_LOW_THRESHOLD (100).
    assert_min_violations(&violations, 1, "fixture exceeds low file size threshold");
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

    // Default threshold (500) — fixture is ~250 lines, should pass.
    assert!(
        violations.is_empty(),
        "Fixture file should be under default threshold"
    );
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

    assert!(
        violations.is_empty(),
        "Clean code should produce no violations, got: {violations:?}"
    );
}
