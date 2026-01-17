//! Tests for Documentation Validation

mod test_utils;

use mcb_validate::{DocumentationValidator, DocumentationViolation};
use tempfile::TempDir;
use test_utils::create_test_crate;

#[test]
fn test_missing_struct_doc() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
pub struct UndocumentedStruct {
    pub field: String,
}
"#,
    );

    let validator = DocumentationValidator::new(temp.path());
    let violations = validator.validate_struct_docs().unwrap();

    assert!(!violations.is_empty(), "Should detect missing struct doc");
    match &violations[0] {
        DocumentationViolation::MissingStructDoc { struct_name, .. } => {
            assert_eq!(struct_name, "UndocumentedStruct");
        }
        _ => panic!("Expected MissingStructDoc"),
    }
}

#[test]
fn test_documented_struct() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
/// A well-documented struct
pub struct DocumentedStruct {
    pub field: String,
}
"#,
    );

    let validator = DocumentationValidator::new(temp.path());
    let violations = validator.validate_struct_docs().unwrap();

    assert!(violations.is_empty(), "Documented structs should pass");
}
