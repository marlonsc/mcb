//! SQLite project repository implementation.

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::project::Project;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam, SqlRow};
use mcb_domain::ports::repositories::ProjectRepository;

use super::row_convert;

/// SQLite-based project repository using the database executor port.
pub struct SqliteProjectRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

impl SqliteProjectRepository {
    /// Create a repository that uses the given executor.
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }

    /// Helper: Query single row and convert to optional entity
    async fn query_one_and_convert<T, F>(
        &self,
        sql: &str,
        params: &[SqlParam],
        convert_fn: F,
        _entity_name: &str,
    ) -> Result<Option<T>>
    where
        F: FnOnce(&dyn SqlRow) -> Result<T>,
    {
        let row = self.executor.query_one(sql, params).await?;
        match row {
            Some(r) => Ok(Some(convert_fn(r.as_ref())?)),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    // Project CRUD
    async fn create(&self, project: &Project) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO projects (id, name, path, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(project.id.clone()),
                    SqlParam::String(project.name.clone()),
                    SqlParam::String(project.path.clone()),
                    SqlParam::I64(project.created_at),
                    SqlParam::I64(project.updated_at),
                ],
            )
            .await
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Project>> {
        self.query_one_and_convert(
            "SELECT * FROM projects WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_convert::row_to_project,
            "project",
        )
        .await
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Project>> {
        self.query_one_and_convert(
            "SELECT * FROM projects WHERE name = ?",
            &[SqlParam::String(name.to_string())],
            row_convert::row_to_project,
            "project",
        )
        .await
    }

    async fn get_by_path(&self, path: &str) -> Result<Option<Project>> {
        self.query_one_and_convert(
            "SELECT * FROM projects WHERE path = ?",
            &[SqlParam::String(path.to_string())],
            row_convert::row_to_project,
            "project",
        )
        .await
    }

    async fn list(&self) -> Result<Vec<Project>> {
        let rows = self
            .executor
            .query_all("SELECT * FROM projects", &[])
            .await?;
        let mut projects = Vec::with_capacity(rows.len());
        for row in rows {
            projects.push(
                row_convert::row_to_project(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode project", e))?,
            );
        }
        Ok(projects)
    }

    async fn update(&self, project: &Project) -> Result<()> {
        self.executor
            .execute(
                "UPDATE projects SET name = ?, path = ?, updated_at = ? WHERE id = ?",
                &[
                    SqlParam::String(project.name.clone()),
                    SqlParam::String(project.path.clone()),
                    SqlParam::I64(project.updated_at),
                    SqlParam::String(project.id.clone()),
                ],
            )
            .await
    }

    async fn delete(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM projects WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await
    }
}
