//! Tests for Implementation Quality Validation
//!
//! Discovery found 9 violations in the full workspace:
//! - Empty method body (2): process(), validate() in my-test
//! - Stub implementation (4): todo!/unimplemented! in my-domain, my-test
//! - Empty catch-all (2): _ => {} match arms in my-domain, my-test
//! - Magic number (1): from organization overlap
//!
//! Uses fixture crates: `my-test`, `my-domain`, `my-infra`

use mcb_validate::ImplementationQualityValidator;

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_implementation_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = ImplementationQualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violation_count(
        &violations,
        9,
        "ImplementationQualityValidator full workspace",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-method tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_empty_method_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has EmptyService with single-line stub methods:
    //   process(&self) -> Result<(), String> { Ok(()) }
    //   validate(&self) -> Result<(), String> { Ok(()) }
    let validator = ImplementationQualityValidator::new(&root);
    let violations = validator.validate_empty_methods().unwrap();

    assert_min_violations(&violations, 1, "empty method stubs in EmptyService");
}

#[test]
fn test_null_provider_exempt_from_empty_checks() {
    let (_temp, root) = with_fixture_crate(INFRA_CRATE);

    // my-infra/src/null.rs has intentionally empty methods (null object pattern)
    let validator = ImplementationQualityValidator::new(&root);
    let violations = validator.validate_empty_methods().unwrap();

    assert_no_violation_from_file(&violations, NULL_RS);
}

#[test]
fn test_todo_macro_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has not_ready_yet() -> todo!("implement this properly")
    let validator = ImplementationQualityValidator::new(&root);
    let violations = validator.validate_stub_macros().unwrap();

    assert_min_violations(&violations, 1, "todo!() macro in not_ready_yet()");
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_constants_file_content_exempt() {
    let (_temp, root) = with_inline_crate(TEST_CRATE, "");
    create_constants_file(&_temp, TEST_CRATE, "pub const MAX_RETRIES: u32 = 100000;\n");

    let validator = ImplementationQualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violation_from_file(&violations, "constants.rs");
}
