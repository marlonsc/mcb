//! Test utilities for mcb-infrastructure integration tests
//!
//! Provides factories and helpers for creating real (not mocked) test contexts
//! that exercise the full DI container and provider stack.

#[allow(dead_code)]
pub mod real_providers;

#[allow(unused_imports)]
pub use real_providers::*;

use mcb_domain::ports::infrastructure::{DatabaseExecutor, SqlParam};

/// Creates a test project row in the in-memory database.
///
/// This is a shared helper used by memory-related tests that need
/// a project to exist before storing observations.
pub async fn create_test_project(executor: &dyn DatabaseExecutor, project_id: &str) {
    let now = chrono::Utc::now().timestamp();
    executor
        .execute(
            "INSERT INTO projects (id, org_id, name, path, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(project_id.to_string()),
                SqlParam::String(mcb_domain::constants::keys::DEFAULT_ORG_ID.to_string()),
                SqlParam::String(project_id.to_string()),
                SqlParam::String("/test".to_string()),
                SqlParam::I64(now),
                SqlParam::I64(now),
            ],
        )
        .await
        .unwrap();
}
