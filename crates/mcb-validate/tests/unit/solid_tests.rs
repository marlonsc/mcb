//! Tests for SOLID Validation
//!
//! Validates `SolidValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use mcb_validate::SolidValidator;

use crate::test_constants::{
    DOMAIN_CRATE, FIXTURE_DOMAIN_SERVICE_PATH, INFRA_CRATE, SERVER_CRATE, TEST_CRATE,
};
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_solid_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = SolidValidator::new(&root);
    let violations = validator.validate_all().unwrap();

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
    let validator = SolidValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(&violations, "Clean SOLID code should produce no violations");
}
