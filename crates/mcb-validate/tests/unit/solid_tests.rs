//! Tests for SOLID Validation
//!
//! Validates `SolidValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use mcb_validate::SolidValidator;

use crate::test_constants::*;
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

    assert_violations_exact(
        &violations,
        &[
            // ── TraitTooLarge (ISP) ─────────────────────────────────────
            ("my-domain/src/domain/service.rs", 137, "TraitTooLarge"),
            // ── PartialTraitImplementation (LSP) ────────────────────────
            (
                "my-domain/src/domain/service.rs",
                158,
                "PartialTraitImplementation",
            ),
            (
                "my-domain/src/domain/service.rs",
                164,
                "PartialTraitImplementation",
            ),
            (
                "my-domain/src/domain/service.rs",
                179,
                "PartialTraitImplementation",
            ),
            (
                "my-domain/src/domain/service.rs",
                182,
                "PartialTraitImplementation",
            ),
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
