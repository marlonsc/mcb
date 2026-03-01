//! Test utilities for mcb-infrastructure
//!
//! ALL shared test helpers live here. No helpers outside this directory.
//! Generic guards (`EnvVarGuard`, `CurrentDirGuard`, `RestoreFileGuard`) and
//! filesystem scanners are centralized in `mcb_domain::test_guards` /
//! `mcb_domain::test_fs_scan`. This module re-exports them.
#![allow(dead_code, clippy::missing_errors_doc, missing_docs, unused_imports)]

use mcb_domain::constants::keys::{DEFAULT_ORG_ID, DEFAULT_ORG_NAME};
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

pub async fn create_test_project(
    db: &DatabaseConnection,
    project_id: &str,
) -> Result<(), sea_orm::DbErr> {
    let now = chrono::Utc::now().timestamp();

    db.execute_raw(Statement::from_sql_and_values(
        db.get_database_backend(),
        "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        [
            DEFAULT_ORG_ID.into(),
            DEFAULT_ORG_NAME.into(),
            "default".into(),
            "{}".into(),
            now.into(),
            now.into(),
        ],
    ))
    .await?;

    db.execute_raw(Statement::from_sql_and_values(
        db.get_database_backend(),
        "INSERT INTO projects (id, org_id, name, path, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        [
            project_id.into(),
            DEFAULT_ORG_ID.into(),
            project_id.into(),
            "/test".into(),
            now.into(),
            now.into(),
        ],
    ))
    .await?;

    Ok(())
}
