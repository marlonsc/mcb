//! Test utilities for mcb-infrastructure
//!
//! ALL shared test helpers live here. No helpers outside this directory.
#![allow(dead_code, clippy::missing_errors_doc, missing_docs)]

#[allow(missing_docs)]
pub mod env_vars;
#[allow(missing_docs)]
pub mod fs_guards;
pub mod real_providers;
pub mod shared_context;
#[allow(missing_docs)]
pub mod workspace;

use sea_orm::{ConnectionTrait, DatabaseConnection};

pub async fn create_test_project(
    db: &DatabaseConnection,
    project_id: &str,
) -> Result<(), sea_orm::DbErr> {
    let now = chrono::Utc::now().timestamp();
    db.execute_unprepared(&format!(
        "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) \
         VALUES ('{org}', '{name}', 'default', '{{}}', {now}, {now})",
        org = mcb_domain::constants::keys::DEFAULT_ORG_ID,
        name = mcb_domain::constants::keys::DEFAULT_ORG_NAME,
    ))
    .await?;
    db.execute_unprepared(&format!(
        "INSERT INTO projects (id, org_id, name, path, created_at, updated_at) \
         VALUES ('{project_id}', '{org}', '{project_id}', '/test', {now}, {now})",
        org = mcb_domain::constants::keys::DEFAULT_ORG_ID,
    ))
    .await?;
    Ok(())
}
