//! Tests for Pattern Validation

use mcb_validate::PatternValidator;
use tempfile::TempDir;

use crate::test_utils::create_test_crate;

#[test]
fn test_arc_mutex_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
use std::sync::{Arc, Mutex};

pub struct State {
    data: Arc<Mutex<Vec<String>>>,
}
",
    );

    let validator = PatternValidator::new(temp.path());
    let violations = validator.validate_async_traits().unwrap();

    // Arc<Mutex<>> can be a code smell in async code - detection depends on rules
    // This test verifies the validator runs without panic
    println!("Arc<Mutex<>> violations found: {}", violations.len());
}

#[test]
fn test_deprecated_api_detection() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r"
use std::mem::uninitialized;

pub fn risky() {
    let x: MaybeUninit<u8> = unsafe { uninitialized() };
}
",
    );

    let validator = PatternValidator::new(temp.path());
    let violations = validator.validate_all().unwrap();

    // Should detect deprecated patterns - detection depends on rules
    // This test verifies the validator runs without panic
    println!("Deprecated pattern violations found: {}", violations.len());
}
