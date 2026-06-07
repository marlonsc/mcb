//! Tests for Implementation Quality Validation
//!
//! Validates `ImplementationQualityValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use crate::utils::test_constants::*;
use crate::utils::*;
use rstest::rstest;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_implementation_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let violations = run_named_validator(&root, "implementation").unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── StubMacro ───────────────────────────────────────────────
            ("my-domain/src/domain/service.rs", 159, "StubMacro"),
            ("my-domain/src/domain/service.rs", 165, "StubMacro"),
            ("my-domain/src/domain/service.rs", 180, "StubMacro"),
            ("my-domain/src/domain/service.rs", 183, "StubMacro"),
            ("my-test/src/lib.rs", 336, "StubMacro"),
            // ── EmptyCatchAll ───────────────────────────────────────────
            ("my-domain/src/domain/service.rs", 130, "EmptyCatchAll"),
            ("my-test/src/lib.rs", 357, "EmptyCatchAll"),
        ],
        "ImplementationQualityValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_clean_implementation_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        "
/// A well-implemented function.
pub fn compute(x: i32) -> i32 {
    x * 2 + 1
}
",
    );
    let violations = run_named_validator(&root, "implementation").unwrap();

    assert_no_violations(
        &violations,
        "Clean implementation should produce no violations",
    );
}
