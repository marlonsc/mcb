//! Tests for Performance Validation
//!
//! Validates `PerformanceValidator` against fixture crates with precise
//! file + line + violation-type assertions.
//!
//! Codes covered: PERF001 (`CloneInLoop`), PERF002 (`AllocationInLoop`),
//! PERF003 (`ArcMutexOveruse`), PERF004 (`InefficientIterator`),
//! PERF005 (`InefficientString`).

use crate::utils::test_constants::*;
use crate::utils::*;
use rstest::rstest;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_performance_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let violations = run_named_validator(&root, "performance").unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── PERF001: CloneInLoop ─────────────────────────────────────
            ("my-test/src/lib.rs", 155, "CloneInLoop"),
            // ── PERF002: AllocationInLoop ────────────────────────────────
            ("my-test/src/lib.rs", 162, "AllocationInLoop"),
            // ── PERF003: ArcMutexOveruse ────────────────────────────────
            ("my-test/src/lib.rs", 169, "ArcMutexOveruse"),
            // ── PERF004: InefficientIterator ────────────────────────────
            ("my-test/src/lib.rs", 174, "InefficientIterator"),
            // ── PERF005: InefficientString ──────────────────────────────
            ("my-test/src/lib.rs", 179, "InefficientString"),
        ],
        "PerformanceValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_clean_performance_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        "
/// Efficient processing without clones or allocations in loops.
pub fn process_items(items: &[String]) -> usize {
    items.len()
}
",
    );
    let violations = run_named_validator(&root, "performance").unwrap();

    assert_no_violations(
        &violations,
        "Clean performance code should produce no violations",
    );
}
