use mcb_domain::constants::keys::{DEFAULT_ORG_ID, DEFAULT_ORG_NAME};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{DatabaseExecutor, SqlParam};

/// Ensure the default organization row exists (FK target for projects).
pub async fn ensure_org_exists(executor: &dyn DatabaseExecutor, timestamp: i64) -> Result<()> {
    executor
        .execute(
            "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(DEFAULT_ORG_ID.to_owned()),
                SqlParam::String(DEFAULT_ORG_NAME.to_owned()),
                SqlParam::String(DEFAULT_ORG_NAME.to_lowercase()),
                SqlParam::String("{}".to_owned()),
                SqlParam::I64(timestamp),
                SqlParam::I64(timestamp),
            ],
        )
        .await
        .map_err(|e| Error::memory_with_source("auto-create default org", e))?;
    Ok(())
}

/// Ensure both the default organization and the given project exist.
///
/// Idempotent — safe to call on every write operation.
pub async fn ensure_org_and_project(
    executor: &dyn DatabaseExecutor,
    project_id: &str,
    timestamp: i64,
) -> Result<()> {
    ensure_org_exists(executor, timestamp).await?;
    executor
        .execute(
            "INSERT OR IGNORE INTO projects (id, org_id, name, path, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(project_id.to_owned()),
                SqlParam::String(DEFAULT_ORG_ID.to_owned()),
                SqlParam::String(format!("Project {project_id}")),
                SqlParam::String(project_id.to_owned()),
                SqlParam::I64(timestamp),
                SqlParam::I64(timestamp),
            ],
        )
        .await
        .map_err(|e| Error::memory_with_source("auto-create project", e))?;
    Ok(())
}
