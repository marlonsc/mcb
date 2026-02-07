//! Tests for KISS (Keep It Simple, Stupid) Validation
//!
//! Uses fixture crate `my-test` which contains realistic too-many-fields
//! (ServerConfig: 8 fields) and too-many-params (initialize_server: 5 params)
//! violations, plus a programmatically generated long function test.

use mcb_validate::{KissValidator, KissViolation};

use crate::test_constants::*;
use crate::test_utils::*;

#[test]
fn test_struct_too_many_fields() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has ServerConfig with 8 fields
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

    // my-test/src/lib.rs has initialize_server with 5 params
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
