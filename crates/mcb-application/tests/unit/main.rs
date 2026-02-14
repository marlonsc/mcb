//! Unit test suite for mcb-application
//!
//! Run with: `cargo test -p mcb-application --test unit`

// Shared test utilities (mock providers for future use)
#[allow(dead_code)]
mod test_utils;

mod constants_tests;
mod decorators;
mod registry_tests;
mod use_cases;
