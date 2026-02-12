//! Tests for Organization Validation
//!
//! Validates `OrganizationValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use mcb_validate::OrganizationValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_organization_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = OrganizationValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violations_exact(
        &violations,
        &[
            // ── MagicNumber ─────────────────────────────────────────────
            (SERVER_CRATE_HANDLER, 82, "MagicNumber"),
            (TEST_CRATE_LIB, 323, "MagicNumber"),
            // ── DomainLayerImplementation ────────────────────────────────
            (DOMAIN_CRATE_SERVICE, 17, "DomainLayerImplementation"),
            (DOMAIN_CRATE_SERVICE, 38, "DomainLayerImplementation"),
            (DOMAIN_CRATE_SERVICE, 49, "DomainLayerImplementation"),
            (DOMAIN_CRATE_SERVICE, 59, "DomainLayerImplementation"),
            (DOMAIN_CRATE_SERVICE, 67, "DomainLayerImplementation"),
            (DOMAIN_CRATE_SERVICE, 94, "DomainLayerImplementation"),
        ],
        "OrganizationValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_clean_organization_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
/// A well-organized module with named constants.
pub const MAX_RETRIES: u32 = 3;
pub fn retry(attempts: u32) -> bool {
    attempts < MAX_RETRIES
}
",
    );
    let validator = OrganizationValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(
        &violations,
        "Clean organized code should produce no violations",
    );
}
