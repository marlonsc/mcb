//! Unit test suite for mcb-validate
//!
//! Run with: `cargo test -p mcb-validate --test unit`

// Shared test utilities (single source — lives outside unit/ dir)
#[path = "../test_utils.rs"]
pub mod test_utils;

// Centralized test constants (shared across unit and integration tests)
#[path = "../test_constants.rs"]
pub mod test_constants;

pub mod ast;
pub mod common;
pub mod engines;
pub mod filters;
pub mod linters;
pub mod rules;
pub mod scan;
pub mod utils;
pub mod validators;

// Legacy/Root
#[path = "lib_tests.rs"]
mod lib_tests;
