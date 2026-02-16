//! Integration test suite for mcb-server
//!
//! Run with: `cargo test -p mcb-server --test integration`

// Integration test modules
mod admin;
mod handlers;
#[path = "utils/mod.rs"]
mod utils;
mod tools;

// Integration helpers - service detection and skip macros
#[path = "integration/utils.rs"]
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

// Golden memory and project workflow E2E tests
#[path = "integration/golden_memory_project_e2e.rs"]
mod golden_memory_project_e2e;

// Hook processor integration tests
#[path = "integration/hooks_integration.rs"]
mod hooks_integration;

// Admin API integration tests
#[path = "integration/admin_api_integration.rs"]
mod admin_api_integration;

// Validation fixes verification (v0.2.0)
#[path = "integration/validation_fixes_e2e.rs"]
mod validation_fixes_e2e;

// Gap fixes verification (GAP-1, GAP-2, GAP-3)
#[path = "integration/gap_fixes_e2e.rs"]
mod gap_fixes_e2e;

// Golden Admin Web UI E2E tests (v0.2.0+)
// CRITICAL: Tests admin_rocket() production config, not just isolated web_rocket()
#[path = "integration/golden_admin_web_e2e.rs"]
mod golden_admin_web_e2e;
