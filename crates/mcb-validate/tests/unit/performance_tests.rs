//! Tests for Performance Validation
//!
//! Discovery found 0 violations in the full workspace.
//! Fixture crates don't currently trigger the performance
//! clone-in-loop or large-return-by-value detectors.

use mcb_validate::performance::PerformanceValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_performance_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = PerformanceValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    // Discovery confirmed 0 violations from fixtures
    assert_violation_count(&violations, 0, "PerformanceValidator full workspace");
}
