//! Tests for Documentation Validation
//!
//! Uses fixture crate `my-test` which contains both documented
//! and undocumented public items (structs, functions).

use mcb_validate::{DocumentationValidator, DocumentationViolation};

use crate::test_constants::TEST_CRATE;
use crate::test_utils::*;

#[test]
fn test_missing_struct_doc() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has UndocumentedStruct without doc comments
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

    // my-test/src/lib.rs has undocumented_function() without doc comments
    let validator = DocumentationValidator::new(&root);
    let violations = validator.validate_pub_item_docs().unwrap();

    assert_has_violation_matching(
        &violations,
        |v| matches!(v, DocumentationViolation::MissingPubItemDoc { item_name, .. } if item_name == "undocumented_function"),
        "MissingPubItemDoc for undocumented_function",
    );
}

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
