//! Integration test suite for mcb-server
//!
//! Run with: `cargo test -p mcb-server --test integration`

// Integration test modules
mod admin;
mod handlers;
mod test_utils;
mod tools;

// Integration helpers - service detection and skip macros
#[path = "integration/helpers.rs"]
mod helpers;

pub use helpers::{
    check_service_available, is_ci, is_milvus_available, is_ollama_available,
    is_postgres_available, is_redis_available,
};

// Golden acceptance tests
#[path = "integration/golden_acceptance_integration.rs"]
mod golden_acceptance_integration;

// Golden MCP tools e2e (no ignore)
#[path = "integration/golden_tools_e2e_integration.rs"]
mod golden_tools_e2e_integration;

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

// Golden E2E complete (no ignore): workflow, index, MCP schema, search
#[path = "integration/golden_e2e_complete_integration.rs"]
mod golden_e2e_complete_integration;

// Golden highlight service tests (Phase 8b)
#[path = "integration/golden_highlight_service_e2e.rs"]
mod golden_highlight_service_e2e;

// Golden tests canonical location (repo root tests/golden) - all included so no tests are discarded
#[path = "../../../tests/golden/test_end_to_end.rs"]
mod golden_test_end_to_end;
#[path = "../../../tests/golden/test_index_repository.rs"]
mod golden_test_index_repository;
#[path = "../../../tests/golden/test_mcp_schemas.rs"]
mod golden_test_mcp_schemas;
#[path = "../../../tests/golden/test_search_validation.rs"]
mod golden_test_search_validation;
