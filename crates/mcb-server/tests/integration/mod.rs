//! Integration test suite for mcb-server
//!
//! Run with: `cargo test -p mcb-server --test integration`

#[path = "../admin/mod.rs"]
mod admin;
#[path = "../handlers/mod.rs"]
mod handlers;
#[path = "../test_utils/mod.rs"]
mod test_utils;
#[path = "../tools/mod.rs"]
mod tools;

// Integration helpers - service detection and skip macros
mod helpers;

pub use helpers::{
    check_service_available, is_ci, is_milvus_available, is_ollama_available,
    is_postgres_available, is_redis_available,
};

// Golden acceptance tests
mod golden_acceptance_integration;

// Golden MCP tools e2e (no ignore)
mod golden_tools_e2e_integration;

// Browse API integration tests
mod browse_api_integration;

// Full-stack DI integration tests
mod full_stack_integration;

// Error recovery integration tests
mod error_recovery_integration;

// Operating modes integration tests (standalone, server, client)
mod operating_modes_integration;

// Golden E2E complete (no ignore): workflow, index, MCP schema, search
mod golden_e2e_complete_integration;

// Golden memory and project workflow E2E tests
mod golden_memory_project_e2e;

// Hook processor integration tests
mod hooks_integration;

// Admin API integration tests
mod admin_api_integration;

// Validation fixes verification (v0.2.0)
mod validation_fixes_e2e;

// Gap fixes verification (GAP-1, GAP-2, GAP-3)
mod gap_fixes_e2e;

// Golden Admin Web UI E2E tests (v0.2.0+)
// CRITICAL: Tests admin_rocket() production config, not just isolated web_rocket()
mod golden_admin_web_e2e;
