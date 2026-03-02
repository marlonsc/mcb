//! Tests for SOLID Validation
//!
//! Validates `SolidValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use crate::utils::test_constants::{
    DOMAIN_CRATE, FIXTURE_DOMAIN_SERVICE_PATH, INFRA_CRATE, SERVER_CRATE, TEST_CRATE,
};
use crate::utils::*;
use rstest::rstest;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_solid_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let violations = run_named_validator(&root, "solid").unwrap();

    let domain_service = format!("{DOMAIN_CRATE}/src/{FIXTURE_DOMAIN_SERVICE_PATH}");
    assert_violations_exact(
        &violations,
        &[
            // ── TraitTooLarge (ISP) ─────────────────────────────────────
            (&domain_service, 137, "TraitTooLarge"),
            // ── PartialTraitImplementation (LSP) ────────────────────────
            (&domain_service, 158, "PartialTraitImplementation"),
            (&domain_service, 164, "PartialTraitImplementation"),
            (&domain_service, 179, "PartialTraitImplementation"),
            (&domain_service, 182, "PartialTraitImplementation"),
        ],
        "SolidValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_clean_solid_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r#"
/// A focused trait following ISP.
pub trait Greeter {
    /// Greets someone.
    fn greet(&self, name: &str) -> String;
}

/// A correct implementation.
pub struct SimpleGreeter;
impl Greeter for SimpleGreeter {
    fn greet(&self, name: &str) -> String {
        format!("Hello, {}!", name)
    }
}
"#,
    );
    let violations = run_named_validator(&root, "solid").unwrap();

    assert_no_violations(&violations, "Clean SOLID code should produce no violations");
}
