//! Tests for Pattern Detection Validation
//!
//! Discovery found 0 violations in the full workspace.
//! The fixture crates don't currently trigger pattern violations
//! (concrete-type DI detection is quite specific).
//!
//! Keeps negative test for Arc<dyn Trait> which should NOT trigger.

use mcb_validate::PatternValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_pattern_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = PatternValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    // Discovery confirmed 0 violations from fixtures
    assert_violation_count(&violations, 0, "PatternValidator full workspace");
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_clean_async_code_no_violation() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
use std::sync::Arc;

pub trait MyService: Send + Sync {
    fn process(&self);
}

pub struct AppState {
    pub service: Arc<dyn MyService>,
}
",
    );

    let validator = PatternValidator::new(&root);
    let violations = validator.validate_trait_based_di().unwrap();

    assert_no_violations(
        &violations,
        "Arc<dyn Trait> should not trigger DI violation",
    );
}
