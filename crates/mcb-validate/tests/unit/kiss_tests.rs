//! Tests for KISS (Keep It Simple, Stupid) Validation
//!
//! Discovery found 2 violations in the full workspace:
//! - StructTooManyFields (ServerParameters in my-test, UserModel in my-domain)
//! - FunctionTooManyParams (initialize_server in my-test, list_users in my-server)
//!
//! Also includes a programmatically generated long-function test and a
//! negative test for acceptable code.

use mcb_validate::{KissValidator, KissViolation};

use crate::test_constants::*;
use crate::test_utils::*;

// ─────────────────────────────────────────────────────────────────────────────
// validate_all() — full workspace
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_kiss_full_workspace() {
    let (_temp, root) =
        with_fixture_workspace(&[TEST_CRATE, DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
    let validator = KissValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    assert_violation_count(&violations, 2, "KissValidator full workspace");
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-method tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_struct_too_many_fields() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has ServerParameters with 8 fields
    let validator = KissValidator::new(&root).with_max_struct_fields(MAX_STRUCT_FIELDS_THRESHOLD);
    let violations = validator.validate_struct_fields().unwrap();

    assert_has_violation_matching(
        &violations,
        |v| matches!(v, KissViolation::StructTooManyFields { .. }),
        "StructTooManyFields",
    );
}

#[test]
fn test_function_too_many_params() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has initialize_server with 6 params
    let validator = KissValidator::new(&root);
    let violations = validator.validate_function_params().unwrap();

    assert_has_violation_matching(
        &violations,
        |v| matches!(v, KissViolation::FunctionTooManyParams { .. }),
        "FunctionTooManyParams",
    );
}

#[test]
fn test_function_too_long() {
    // Generate a long function dynamically (hard to have in a fixture file)
    let long_function = format!(
        r"
pub fn long_function() {{
{}
}}
",
        (0..LONG_FUNCTION_LINE_COUNT)
            .map(|i| format!("    let x{i} = {i};"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let (_temp, root) = with_inline_crate(TEST_CRATE, &long_function);

    let validator = KissValidator::new(&root);
    let violations = validator.validate_function_length().unwrap();

    assert_has_violation_matching(
        &violations,
        |v| matches!(v, KissViolation::FunctionTooLong { .. }),
        "FunctionTooLong",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Negative test
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_acceptable_struct_no_violations() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
pub struct SmallStruct {
    field1: String,
    field2: String,
    field3: String,
}
",
    );

    let validator = KissValidator::new(&root);
    let violations = validator.validate_struct_fields().unwrap();

    assert_no_violations(&violations, "3-field struct should pass");
}
