//! Tests for Performance Validation
//!
//! Validates `PerformanceValidator` against fixture crates with precise
//! file + line + violation-type assertions.
//!
//! Codes covered: PERF001 (CloneInLoop), PERF002 (AllocationInLoop),
//! PERF003 (ArcMutexOveruse), PERF004 (InefficientIterator),
//! PERF005 (InefficientString).

use mcb_validate::PerformanceValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_performance_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = PerformanceValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── PERF001: CloneInLoop ─────────────────────────────────────
            (TEST_CRATE_LIB, 155, "CloneInLoop"),
            // ── PERF002: AllocationInLoop ────────────────────────────────
            (TEST_CRATE_LIB, 162, "AllocationInLoop"),
            // ── PERF003: ArcMutexOveruse ────────────────────────────────
            (TEST_CRATE_LIB, 169, "ArcMutexOveruse"),
            // ── PERF004: InefficientIterator ────────────────────────────
            (TEST_CRATE_LIB, 174, "InefficientIterator"),
            // ── PERF005: InefficientString ──────────────────────────────
            (TEST_CRATE_LIB, 179, "InefficientString"),
        ],
        "PerformanceValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_clean_performance_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r#"
/// Efficient processing without clones or allocations in loops.
pub fn process_items(items: &[String]) -> usize {
    items.len()
}
"#,
    );
    let validator = PerformanceValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(
        &violations,
        "Clean performance code should produce no violations",
    );
}
