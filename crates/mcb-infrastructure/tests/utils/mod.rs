//! Test utilities for mcb-infrastructure
//!
//! ALL shared test helpers live here. No helpers outside this directory.
#![allow(dead_code)]

pub mod real_providers;
pub mod shared_context;

use mcb_domain::ports::{DatabaseExecutor, SqlParam};

/// Creates a test project row in the in-memory database.
///
/// This is a shared helper used by memory-related tests that need
/// a project to exist before storing observations.
///
/// # Errors
///
/// Returns an error if the database INSERT statements fail.
pub async fn create_test_project(
    executor: &dyn DatabaseExecutor,
    project_id: &str,
) -> mcb_domain::error::Result<()> {
    let now = chrono::Utc::now().timestamp();
    // Ensure default organization exists (FK: projects.org_id → organizations.id)
    executor
        .execute(
            "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(mcb_domain::constants::keys::DEFAULT_ORG_ID.to_owned()),
                SqlParam::String(mcb_domain::constants::keys::DEFAULT_ORG_NAME.to_owned()),
                SqlParam::String("default".to_owned()),
                SqlParam::String("{}".to_owned()),
                SqlParam::I64(now),
                SqlParam::I64(now),
            ],
        )
        .await?;
    executor
        .execute(
            "INSERT INTO projects (id, org_id, name, path, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(project_id.to_owned()),
                SqlParam::String(mcb_domain::constants::keys::DEFAULT_ORG_ID.to_owned()),
                SqlParam::String(project_id.to_owned()),
                SqlParam::String("/test".to_owned()),
                SqlParam::I64(now),
                SqlParam::I64(now),
            ],
        )
        .await?;
    Ok(())
}
