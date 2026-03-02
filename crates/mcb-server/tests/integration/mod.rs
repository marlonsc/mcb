//! Integration tests — `cargo test -p mcb-server --test integration`

/// Shared test utilities.
#[path = "../utils/mod.rs"]
pub mod utils;

/// Handler integration tests.
pub mod handlers;

mod error_recovery_integration;
mod error_shape_tests;
mod form_deserialization_test;
mod full_stack_integration;
mod golden_acceptance_integration;
mod hooks_integration;

mod admin_api_tests;
mod auto_context_tests;
