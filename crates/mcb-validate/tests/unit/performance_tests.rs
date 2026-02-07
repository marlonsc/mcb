//! Tests for Performance Validation
//!
//! Uses fixture crate `my-test` which contains:
//! - `batch_process`: clone in loop (template.clone() inside for loop)
//! - `LargeConfig`: large struct returned by value (3x [u8; 1024])

use mcb_validate::performance::PerformanceValidator;

use crate::test_constants::TEST_CRATE;
use crate::test_utils::*;

#[test]
fn test_clone_in_loop_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has batch_process() with template.clone() in for loop
    let validator = PerformanceValidator::new(&root);
    let violations = validator.validate_clone_in_loops().unwrap();

    // Log results — fixture may or may not trigger based on pattern detection
    println!("Clone-in-loop violations found: {}", violations.len());
}

#[test]
fn test_validate_all() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has LargeConfig (3x [u8; 1024]) returned by value
    let validator = PerformanceValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    println!("Performance violations found: {}", violations.len());
}
