//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md#database)
//!
//! `SQLite` Project Repository
//!
//! # Overview
//! The `SqliteProjectRepository` handles the persistence of project entities, which represent
//! distinct codebases or modules within an organization. It supports hierarchical organization
//! via `org_id` and tracks project metadata like paths and update timestamps.
//!
//! # Responsibilities
//! - **Project Registry**: Storing the definition of all projects in the system.
//! - **Lookup Logic**: Finding projects by ID, name, or file path.
//! - **CRUD Operations**: Creating, updating, and deleting project records.

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::project::Project;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::ProjectRepository;
use mcb_domain::ports::{DatabaseExecutor, SqlParam};

use super::row_convert;
use crate::utils::sqlite::query as query_helpers;

/// SQLite-based implementation of the `ProjectRepository`.
///
/// Implements standard CRUD for the `projects` table.
/// Provides efficient lookups by `path` and `name` to support project detection and resolution logic.
pub struct SqliteProjectRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

const INSERT_PROJECT_SQL: &str = "INSERT INTO projects (id, org_id, name, path, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)";

fn project_insert_params(project: &Project) -> [SqlParam; 6] {
    [
        SqlParam::String(project.id.clone()),
        SqlParam::String(project.org_id.clone()),
        SqlParam::String(project.name.clone()),
        SqlParam::String(project.path.clone()),
        SqlParam::I64(project.created_at),
        SqlParam::I64(project.updated_at),
    ]
}

impl SqliteProjectRepository {
    /// Create a repository that uses the given executor.
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }
}

#[async_trait]
/// Persistent project repository using `SQLite`.
impl ProjectRepository for SqliteProjectRepository {
    /// Creates a new project.
    async fn create(&self, project: &Project) -> Result<()> {
        let params = project_insert_params(project);
        query_helpers::execute(&self.executor, INSERT_PROJECT_SQL, &params).await
    }

    /// Retrieves a project by ID.
    async fn get_by_id(&self, org_id: &str, id: &str) -> Result<Project> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM projects WHERE org_id = ? AND id = ? LIMIT 1",
            &[
                SqlParam::String(org_id.to_owned()),
                SqlParam::String(id.to_owned()),
            ],
            row_convert::row_to_project,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Project {id}")))
    }

    /// Retrieves a project by name.
    async fn get_by_name(&self, org_id: &str, name: &str) -> Result<Project> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM projects WHERE org_id = ? AND name = ? LIMIT 1",
            &[
                SqlParam::String(org_id.to_owned()),
                SqlParam::String(name.to_owned()),
            ],
            row_convert::row_to_project,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Project {name}")))
    }

    /// Retrieves a project by path.
    async fn get_by_path(&self, org_id: &str, path: &str) -> Result<Project> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM projects WHERE org_id = ? AND path = ? LIMIT 1",
            &[
                SqlParam::String(org_id.to_owned()),
                SqlParam::String(path.to_owned()),
            ],
            row_convert::row_to_project,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Project {path}")))
    }

    /// Lists all projects in an organization.
    async fn list(&self, org_id: &str) -> Result<Vec<Project>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM projects WHERE org_id = ?",
            &[SqlParam::String(org_id.to_owned())],
            row_convert::row_to_project,
            "project",
        )
        .await
    }

    /// Updates an existing project.
    async fn update(&self, project: &Project) -> Result<()> {
        self.executor
            .execute(
                "UPDATE projects SET name = ?, path = ?, updated_at = ? WHERE org_id = ? AND id = ?",
                &[
                    SqlParam::String(project.name.clone()),
                    SqlParam::String(project.path.clone()),
                    SqlParam::I64(project.updated_at),
                    SqlParam::String(project.org_id.clone()),
                    SqlParam::String(project.id.clone()),
                ],
            )
            .await
    }

    /// Deletes a project.
    async fn delete(&self, org_id: &str, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM projects WHERE org_id = ? AND id = ?",
                &[
                    SqlParam::String(org_id.to_owned()),
                    SqlParam::String(id.to_owned()),
                ],
            )
            .await
    }
}
