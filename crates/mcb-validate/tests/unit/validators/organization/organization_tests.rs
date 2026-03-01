//! Tests for Organization Validation
//!
//! Validates `OrganizationValidator` against fixture crates with precise
//! file + line + violation-type assertions.

use mcb_validate::OrganizationValidator;
use mcb_validate::{OrganizationViolation, Severity, Violation};

use crate::utils::test_constants::*;
use crate::utils::*;
use rstest::rstest;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
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
            ("my-server/src/handlers/user_handler.rs", 82, "MagicNumber"),
            ("my-test/src/lib.rs", 323, "MagicNumber"),
            // ── DomainLayerImplementation ────────────────────────────────
            (
                "my-domain/src/domain/service.rs",
                17,
                "DomainLayerImplementation",
            ),
            (
                "my-domain/src/domain/service.rs",
                38,
                "DomainLayerImplementation",
            ),
            (
                "my-domain/src/domain/service.rs",
                49,
                "DomainLayerImplementation",
            ),
            (
                "my-domain/src/domain/service.rs",
                59,
                "DomainLayerImplementation",
            ),
            (
                "my-domain/src/domain/service.rs",
                67,
                "DomainLayerImplementation",
            ),
            (
                "my-domain/src/domain/service.rs",
                94,
                "DomainLayerImplementation",
            ),
        ],
        "OrganizationValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[rstest]
#[test]
fn test_clean_organization_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        "
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

#[rstest]
#[test]
fn test_organization_violation_severity_is_non_recursive() {
    let violation = OrganizationViolation::MagicNumber {
        file: std::path::PathBuf::from("dummy.rs"),
        line: 1,
        value: "99999".to_owned(),
        context: "let n = 99999;".to_owned(),
        suggestion: "Use constant".to_owned(),
        severity: Severity::Info,
    };

    assert_eq!(Violation::severity(&violation), Severity::Info);
}
