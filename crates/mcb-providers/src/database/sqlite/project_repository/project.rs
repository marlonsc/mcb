use crate::database::sqlite::row_convert;
use mcb_domain::entities::project::Project;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam};
use std::sync::Arc;
use tracing::debug;

pub(crate) async fn create(executor: &Arc<dyn DatabaseExecutor>, project: &Project) -> Result<()> {
    let params = [
        SqlParam::String(project.id.clone()),
        SqlParam::String(project.name.clone()),
        SqlParam::String(project.path.clone()),
        SqlParam::I64(project.created_at),
        SqlParam::I64(project.updated_at),
    ];

    executor
        .execute(
            r"
            INSERT INTO projects (id, name, path, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ",
            &params,
        )
        .await?;

    debug!("Created project: {}", project.id);
    Ok(())
}

pub(crate) async fn get_by_id(
    executor: &Arc<dyn DatabaseExecutor>,
    id: &str,
) -> Result<Option<Project>> {
    let row = executor
        .query_one(
            "SELECT * FROM projects WHERE id = ?",
            &[SqlParam::String(id.to_string())],
        )
        .await?;

    match row {
        Some(r) => Ok(Some(
            row_convert::row_to_project(r.as_ref())
                .map_err(|e| Error::memory_with_source("decode project row", e))?,
        )),
        None => Ok(None),
    }
}

pub(crate) async fn get_by_name(
    executor: &Arc<dyn DatabaseExecutor>,
    name: &str,
) -> Result<Option<Project>> {
    let row = executor
        .query_one(
            "SELECT * FROM projects WHERE name = ?",
            &[SqlParam::String(name.to_string())],
        )
        .await?;

    match row {
        Some(r) => Ok(Some(
            row_convert::row_to_project(r.as_ref())
                .map_err(|e| Error::memory_with_source("decode project row", e))?,
        )),
        None => Ok(None),
    }
}

pub(crate) async fn get_by_path(
    executor: &Arc<dyn DatabaseExecutor>,
    path: &str,
) -> Result<Option<Project>> {
    let row = executor
        .query_one(
            "SELECT * FROM projects WHERE path = ?",
            &[SqlParam::String(path.to_string())],
        )
        .await?;

    match row {
        Some(r) => Ok(Some(
            row_convert::row_to_project(r.as_ref())
                .map_err(|e| Error::memory_with_source("decode project row", e))?,
        )),
        None => Ok(None),
    }
}

pub(crate) async fn list(executor: &Arc<dyn DatabaseExecutor>) -> Result<Vec<Project>> {
    let rows = executor
        .query_all("SELECT * FROM projects ORDER BY created_at DESC", &[])
        .await?;

    let mut projects = Vec::with_capacity(rows.len());
    for row in rows {
        projects.push(
            row_convert::row_to_project(row.as_ref())
                .map_err(|e| Error::memory_with_source("decode project row", e))?,
        );
    }
    Ok(projects)
}

pub(crate) async fn update(executor: &Arc<dyn DatabaseExecutor>, project: &Project) -> Result<()> {
    let params = [
        SqlParam::String(project.name.clone()),
        SqlParam::String(project.path.clone()),
        SqlParam::I64(project.updated_at),
        SqlParam::String(project.id.clone()),
    ];

    executor
        .execute(
            r"
            UPDATE projects
            SET name = ?, path = ?, updated_at = ?
            WHERE id = ?
            ",
            &params,
        )
        .await?;

    debug!("Updated project: {}", project.id);
    Ok(())
}

pub(crate) async fn delete(executor: &Arc<dyn DatabaseExecutor>, id: &str) -> Result<()> {
    executor
        .execute(
            "DELETE FROM projects WHERE id = ?",
            &[SqlParam::String(id.to_string())],
        )
        .await?;

    debug!("Deleted project: {}", id);
    Ok(())
}
