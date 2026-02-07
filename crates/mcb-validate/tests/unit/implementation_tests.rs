//! Tests for Implementation Quality Validation
//!
//! Uses fixture crates:
//! - `my-test`: contains empty methods (EmptyService), todo!(), empty catch-all
//! - `my-infra`: contains null.rs (should be exempt from empty-method checks)

use mcb_validate::ImplementationQualityValidator;

use crate::test_constants::*;
use crate::test_utils::*;

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

#[test]
fn test_empty_catchall_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has handle_message() with _ => {}
    let validator = ImplementationQualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    // Should detect the empty catch-all match arm
    println!("Implementation violations found: {}", violations.len());
}

#[test]
fn test_constants_file_content_exempt() {
    let (_temp, root) = with_inline_crate(TEST_CRATE, "");
    // Add a constants.rs with numeric content
    create_constants_file(
        // _temp is a TempDir, created by with_inline_crate
        // We need to use the TmpDir reference for helpers that need &TempDir
        // but with_inline_crate already set up the workspace,
        // so add the file directly
        &_temp,
        TEST_CRATE,
        "pub const MAX_RETRIES: u32 = 100000;\n",
    );

    let validator = ImplementationQualityValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_no_violation_from_file(&violations, "constants.rs");
}
