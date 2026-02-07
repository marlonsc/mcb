//! Tests for Documentation Validation
//!
//! Discovery found 8 violations in the full workspace:
//! - Missing pub-item docs across `my-test`, `my-domain` and `my-server`

use mcb_validate::{DocumentationValidator, DocumentationViolation};

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_documentation_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = DocumentationValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violation_count(&violations, 8, "DocumentationValidator full workspace");
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-item tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_missing_struct_doc() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = DocumentationValidator::new(&root);
    let violations = validator.validate_pub_item_docs().unwrap();

    assert_has_violation_matching(
        &violations,
        |v| matches!(v, DocumentationViolation::MissingPubItemDoc { item_name, .. } if item_name == "UndocumentedStruct"),
        "MissingPubItemDoc for UndocumentedStruct",
    );
}

#[test]
fn test_missing_function_doc() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);
    let validator = DocumentationValidator::new(&root);
    let violations = validator.validate_pub_item_docs().unwrap();

    assert_has_violation_matching(
        &violations,
        |v| matches!(v, DocumentationViolation::MissingPubItemDoc { item_name, .. } if item_name == "undocumented_function"),
        "MissingPubItemDoc for undocumented_function",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_documented_struct_no_violation() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
/// A well-documented struct
pub struct DocumentedStruct {
    pub field: String,
}
",
    );

    let validator = DocumentationValidator::new(&root);
    let violations = validator.validate_pub_item_docs().unwrap();

    assert_no_violations(&violations, "Documented structs should pass");
}
