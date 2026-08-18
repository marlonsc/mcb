//! Integration tests — `cargo test -p mcb-server --test integration`

// linkme force-link only — DO NOT use for type/function imports (CA019 enforced)
extern crate mcb_infrastructure;
extern crate mcb_providers;
extern crate mcb_validate;

/// Shared test utilities.
#[path = "../utils/mod.rs"]
pub mod utils;

/// Handler integration tests.
pub mod handlers;

mod error_recovery_integration;
mod error_shape_tests_integration;
mod form_deserialization_test_integration;
mod full_stack_integration;
mod golden_acceptance_integration;
mod hooks_integration;

mod admin_api_tests_integration;
mod admin_http_tests_integration;
mod auto_context_tests_integration;
mod http_mcp_tests_integration;
