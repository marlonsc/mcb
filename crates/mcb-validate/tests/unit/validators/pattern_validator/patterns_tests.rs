//! Tests for Pattern Validation
//!
//! Validates `PatternValidator` against fixture crates with precise
//! file + line + violation-type assertions.
//!
//! Codes covered: PAT001 (`ConcreteTypeInDi`), PAT004 (`RawResultType`).

use mcb_validate::PatternValidator;

use crate::utils::test_constants::*;
use crate::utils::*;
use rstest::rstest;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_patterns_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = PatternValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── PAT001: ConcreteTypeInDi ────────────────────────────────
            ("my-test/src/lib.rs", 191, "ConcreteTypeInDi"),
            // ── PAT004: RawResultType ───────────────────────────────────
            ("my-test/src/lib.rs", 195, "RawResultType"),
        ],
        "PatternValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_clean_patterns_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        "
/// Clean DI using trait objects.
pub trait CacheService {
    fn get(&self, key: &str) -> Option<String>;
}
",
    );
    let validator = PatternValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(
        &violations,
        "Clean pattern code should produce no violations",
    );
}
