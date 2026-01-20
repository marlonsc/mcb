//! Integration test suite for mcb-server
//!
//! Run with: `cargo test -p mcb-server --test integration`

// Integration test modules
mod admin;
mod handlers;
mod test_utils;
mod tools;

// Golden acceptance tests
#[path = "integration/golden_acceptance.rs"]
mod golden_acceptance;

// Browse API tests
#[path = "integration/browse_api_test.rs"]
mod browse_api_test;
