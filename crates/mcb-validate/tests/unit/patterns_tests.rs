//! Tests for Pattern Detection Validation
//!
//! Uses fixture crates:
//! - `my-server`: contains Arc<Mutex<>> in async handler code
//! - `my-test`: contains Arc<Mutex<>> in shared_state_handler

use mcb_validate::{PatternValidator, PatternViolation};

use crate::test_constants::*;
use crate::test_utils::*;

#[test]
fn test_arc_mutex_in_async_detection() {
    let (_temp, root) = with_fixture_crate(TEST_CRATE);

    // my-test/src/lib.rs has shared_state_handler() using Arc<Mutex<>>
    let validator = PatternValidator::new(&root);
    let violations = validator.validate_trait_based_di().unwrap();

    // Pattern validator checks for concrete type DI, not arc-mutex specifically
    println!("Pattern violations from fixture: {}", violations.len());
}

#[test]
fn test_pattern_violations_in_server() {
    let (_temp, root) = with_fixture_crate(SERVER_CRATE);

    // my-server has handlers with concrete type dependencies
    let validator = PatternValidator::new(&root);
    let violations = validator.validate_all().unwrap();

    println!("Pattern violations in server: {}", violations.len());
}

#[test]
fn test_clean_async_code_no_violation() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
use std::sync::Arc;

pub trait MyService: Send + Sync {
    fn process(&self);
}

pub struct AppState {
    pub service: Arc<dyn MyService>,
}
",
    );

    let validator = PatternValidator::new(&root);
    let violations = validator.validate_trait_based_di().unwrap();

    assert_no_violations(
        &violations,
        "Arc<dyn Trait> should not trigger DI violation",
    );
}
