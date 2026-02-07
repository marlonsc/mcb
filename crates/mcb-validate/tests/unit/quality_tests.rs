//! Tests for Quality Validation
//!
//! Validates each method of `QualityValidator` against the `my-test` fixture
//! crate, which contains realistic violations embedded in plausible code:
//!
//! | Method                             | Fixture violation                                |
//! |------------------------------------|--------------------------------------------------|
//! | `validate_no_unwrap_expect()`      | `load_config()`: `.unwrap()`, `.expect()`        |
//! | `find_todo_comments()`             | `load_config()`: TODO, `merge_configs()`: FIXME  |
//! | `validate_no_panic()`              | `validate_critical_config()`: `panic!()`         |
//! | `validate_dead_code_annotations()` | `InternalCache`: `#[allow(dead_code)]`           |
//! | `validate_file_sizes()`            | Tested with low threshold on fixture file        |
//! | `validate_all()`                   | Aggregates all of the above                      |

use mcb_validate::{QualityValidator, QualityViolation};

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_no_unwrap_expect()
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_unwrap_expect_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_no_unwrap_expect().unwrap();

    // my-test/src/lib.rs → load_config():
    //   L17: std::fs::read_to_string(path).unwrap()
    //   L19: serde_json::from_str(&content).expect("invalid config JSON")
    // my-test/src/lib.rs → shared_state_handler():
    //   data.lock().unwrap()
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
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = QualityValidator::new(&root);
    let violations = validator.find_todo_comments().unwrap();

    // load_config():   "TODO: Add validation for config schema"
    // merge_configs(): "FIXME: This doesn't handle nested merges"
    assert_min_violations(&violations, 2, "TODO and FIXME comments");
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

    // #[allow(dead_code)] on InternalCache struct and unused_helper() fn
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
// validate_all()
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_validate_all_aggregates_checks() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = QualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    // Aggregates: unwrap, expect, panic, TODO, FIXME, dead_code
    assert_min_violations(&violations, 5, "validate_all aggregate");
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
