//! Tests for Documentation Validation
//!
//! Validates `DocumentationValidator` against fixture crates with precise
//! file + line + violation-type assertions.
//!
//! Note: `MissingModuleDoc` violations have no line field, so line=0
//! is used (skips line check).

use mcb_validate::DocumentationValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace, precise assertions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_documentation_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = DocumentationValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violations_exact(
        &violations,
        &[
            // MissingModuleDoc — no line field, use 0 to skip check
            ("my-server/src/lib.rs", 0, "MissingModuleDoc"),
            ("my-server/src/handlers/mod.rs", 0, "MissingModuleDoc"),
            ("my-domain/src/lib.rs", 0, "MissingModuleDoc"),
            ("my-domain/src/domain/mod.rs", 0, "MissingModuleDoc"),
            // MissingExampleCode — has line field
            ("my-domain/src/domain/service.rs", 137, "MissingExampleCode"),
            // MissingPubItemDoc — has line field
            ("my-domain/src/domain/model.rs", 16, "MissingPubItemDoc"),
            ("my-test/src/lib.rs", 136, "MissingPubItemDoc"),
            ("my-test/src/lib.rs", 141, "MissingPubItemDoc"),
        ],
        "DocumentationValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test: clean code
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_well_documented_code_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r#"
//! Well-documented crate.
/// A well-documented public function.
///
/// # Examples
/// ```
/// assert_eq!(my_crate::add(1, 2), 3);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    );
    let validator = DocumentationValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violations(
        &violations,
        "Well-documented code should produce no violations",
    );
}
