//! Integration test suite for mcb-server
//!
//! Run with: `cargo test -p mcb-server --test integration`

// Integration test modules
mod admin;
mod handlers;
mod test_utils;
mod tools;

// Integration helpers - service detection and skip macros
#[path = "integration_helpers.rs"]
mod integration_helpers;

pub use integration_helpers::{
    check_service_available, is_milvus_available, is_ollama_available, is_postgres_available,
    is_redis_available,
};

// Golden acceptance tests
#[path = "integration/golden_acceptance_integration.rs"]
mod golden_acceptance_integration;

// Browse API integration tests
#[path = "integration/browse_api_integration.rs"]
mod browse_api_integration;

// Full-stack DI integration tests
#[path = "integration/full_stack_integration.rs"]
mod full_stack_integration;

// Error recovery integration tests
#[path = "integration/error_recovery_integration.rs"]
mod error_recovery_integration;

// Operating modes integration tests (standalone, server, client)
#[path = "integration/operating_modes_integration.rs"]
mod operating_modes_integration;

// Stdio transport integration tests (Claude Code compatibility)
#[path = "integration/stdio_transport_integration.rs"]
mod stdio_transport_integration;
