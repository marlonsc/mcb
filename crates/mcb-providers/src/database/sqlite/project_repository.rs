//! SQLite project repository using the domain port [`DatabaseExecutor`].
//!
//! Implements [`ProjectRepository`] via [`DatabaseExecutor`]; no direct sqlx in this module.

use super::row_convert;

use async_trait::async_trait;
use mcb_domain::entities::project::{
    Project, ProjectDecision, ProjectDependency, ProjectIssue, ProjectPhase,
};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::{IssueFilter, ProjectRepository};
use std::sync::Arc;
use tracing::debug;

/// SQLite-based project repository using the database executor port.
pub struct SqliteProjectRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

impl SqliteProjectRepository {
    /// Create a repository that uses the given executor (from provider factory).
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    // ========== Project CRUD ==========

    async fn create(&self, project: &Project) -> Result<()> {
        let params = [
            SqlParam::String(project.id.clone()),
            SqlParam::String(project.name.clone()),
            SqlParam::String(project.path.clone()),
            SqlParam::I64(project.created_at),
            SqlParam::I64(project.updated_at),
        ];

        self.executor
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

    async fn get_by_id(&self, id: &str) -> Result<Option<Project>> {
        let row = self
            .executor
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

    async fn get_by_name(&self, name: &str) -> Result<Option<Project>> {
        let row = self
            .executor
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

    async fn get_by_path(&self, path: &str) -> Result<Option<Project>> {
        let row = self
            .executor
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

    async fn list(&self) -> Result<Vec<Project>> {
        let rows = self
            .executor
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

    async fn update(&self, project: &Project) -> Result<()> {
        let params = [
            SqlParam::String(project.name.clone()),
            SqlParam::String(project.path.clone()),
            SqlParam::I64(project.updated_at),
            SqlParam::String(project.id.clone()),
        ];

        self.executor
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

    async fn delete(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM projects WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await?;

        debug!("Deleted project: {}", id);
        Ok(())
    }

    // ========== Phase operations ==========

    async fn create_phase(&self, phase: &ProjectPhase) -> Result<()> {
        let params = [
            SqlParam::String(phase.id.clone()),
            SqlParam::String(phase.project_id.clone()),
            SqlParam::String(phase.name.clone()),
            SqlParam::String(phase.description.clone()),
            SqlParam::I64(phase.sequence as i64),
            SqlParam::String(phase.status.as_str().to_string()),
            phase.started_at.map_or(SqlParam::Null, SqlParam::I64),
            phase.completed_at.map_or(SqlParam::Null, SqlParam::I64),
            SqlParam::I64(phase.created_at),
            SqlParam::I64(phase.updated_at),
        ];

        self.executor
            .execute(
                r"
                INSERT INTO project_phases (
                    id, project_id, name, description, sequence, status,
                    started_at, completed_at, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ",
                &params,
            )
            .await?;

        debug!("Created phase: {}", phase.id);
        Ok(())
    }

    async fn get_phase(&self, id: &str) -> Result<Option<ProjectPhase>> {
        let row = self
            .executor
            .query_one(
                "SELECT * FROM project_phases WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await?;

        match row {
            Some(r) => Ok(Some(
                row_convert::row_to_project_phase(r.as_ref())
                    .map_err(|e| Error::memory_with_source("decode project phase row", e))?,
            )),
            None => Ok(None),
        }
    }

    async fn update_phase(&self, phase: &ProjectPhase) -> Result<()> {
        let params = [
            SqlParam::String(phase.name.clone()),
            SqlParam::String(phase.description.clone()),
            SqlParam::I64(phase.sequence as i64),
            SqlParam::String(phase.status.as_str().to_string()),
            phase.started_at.map_or(SqlParam::Null, SqlParam::I64),
            phase.completed_at.map_or(SqlParam::Null, SqlParam::I64),
            SqlParam::I64(phase.updated_at),
            SqlParam::String(phase.id.clone()),
        ];

        self.executor
            .execute(
                r"
                UPDATE project_phases
                SET name = ?, description = ?, sequence = ?, status = ?,
                    started_at = ?, completed_at = ?, updated_at = ?
                WHERE id = ?
                ",
                &params,
            )
            .await?;

        debug!("Updated phase: {}", phase.id);
        Ok(())
    }

    async fn list_phases(&self, project_id: &str) -> Result<Vec<ProjectPhase>> {
        let rows = self
            .executor
            .query_all(
                "SELECT * FROM project_phases WHERE project_id = ? ORDER BY sequence ASC",
                &[SqlParam::String(project_id.to_string())],
            )
            .await?;

        let mut phases = Vec::with_capacity(rows.len());
        for row in rows {
            phases.push(
                row_convert::row_to_project_phase(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode project phase row", e))?,
            );
        }
        Ok(phases)
    }

    // ========== Issue operations ==========

    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()> {
        let labels_json = serde_json::to_string(&issue.labels)
            .map_err(|e| Error::memory_with_source("serialize issue labels", e))?;

        let params = [
            SqlParam::String(issue.id.clone()),
            SqlParam::String(issue.project_id.clone()),
            issue
                .phase_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            SqlParam::String(issue.title.clone()),
            SqlParam::String(issue.description.clone()),
            SqlParam::String(issue.issue_type.as_str().to_string()),
            SqlParam::String(issue.status.as_str().to_string()),
            SqlParam::I64(issue.priority as i64),
            issue
                .assignee
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            SqlParam::String(labels_json),
            SqlParam::I64(issue.created_at),
            SqlParam::I64(issue.updated_at),
            issue.closed_at.map_or(SqlParam::Null, SqlParam::I64),
        ];

        self.executor
            .execute(
                r"
                INSERT INTO project_issues (
                    id, project_id, phase_id, title, description, issue_type, status,
                    priority, assignee, labels, created_at, updated_at, closed_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ",
                &params,
            )
            .await?;

        debug!("Created issue: {}", issue.id);
        Ok(())
    }

    async fn get_issue(&self, id: &str) -> Result<Option<ProjectIssue>> {
        let row = self
            .executor
            .query_one(
                "SELECT * FROM project_issues WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await?;

        match row {
            Some(r) => Ok(Some(
                row_convert::row_to_project_issue(r.as_ref())
                    .map_err(|e| Error::memory_with_source("decode project issue row", e))?,
            )),
            None => Ok(None),
        }
    }

    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()> {
        let labels_json = serde_json::to_string(&issue.labels)
            .map_err(|e| Error::memory_with_source("serialize issue labels", e))?;

        let params = [
            issue
                .phase_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            SqlParam::String(issue.title.clone()),
            SqlParam::String(issue.description.clone()),
            SqlParam::String(issue.issue_type.as_str().to_string()),
            SqlParam::String(issue.status.as_str().to_string()),
            SqlParam::I64(issue.priority as i64),
            issue
                .assignee
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            SqlParam::String(labels_json),
            SqlParam::I64(issue.updated_at),
            issue.closed_at.map_or(SqlParam::Null, SqlParam::I64),
            SqlParam::String(issue.id.clone()),
        ];

        self.executor
            .execute(
                r"
                UPDATE project_issues
                SET phase_id = ?, title = ?, description = ?, issue_type = ?, status = ?,
                    priority = ?, assignee = ?, labels = ?, updated_at = ?, closed_at = ?
                WHERE id = ?
                ",
                &params,
            )
            .await?;

        debug!("Updated issue: {}", issue.id);
        Ok(())
    }

    async fn list_issues(&self, project_id: &str) -> Result<Vec<ProjectIssue>> {
        let rows = self
            .executor
            .query_all(
                "SELECT * FROM project_issues WHERE project_id = ? ORDER BY created_at DESC",
                &[SqlParam::String(project_id.to_string())],
            )
            .await?;

        let mut issues = Vec::with_capacity(rows.len());
        for row in rows {
            issues.push(
                row_convert::row_to_project_issue(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode project issue row", e))?,
            );
        }
        Ok(issues)
    }

    async fn filter_issues(&self, filter: &IssueFilter) -> Result<Vec<ProjectIssue>> {
        let mut sql = String::from("SELECT * FROM project_issues WHERE 1=1");
        let mut params: Vec<SqlParam> = Vec::new();

        if let Some(project_id) = &filter.project_id {
            sql.push_str(" AND project_id = ?");
            params.push(SqlParam::String(project_id.clone()));
        }
        if let Some(phase_id) = &filter.phase_id {
            sql.push_str(" AND phase_id = ?");
            params.push(SqlParam::String(phase_id.clone()));
        }
        if let Some(issue_type) = &filter.issue_type {
            sql.push_str(" AND issue_type = ?");
            params.push(SqlParam::String(issue_type.as_str().to_string()));
        }
        if let Some(status) = &filter.status {
            sql.push_str(" AND status = ?");
            params.push(SqlParam::String(status.as_str().to_string()));
        }
        if let Some(priority) = filter.priority {
            sql.push_str(" AND priority = ?");
            params.push(SqlParam::I64(priority as i64));
        }
        if let Some(assignee) = &filter.assignee {
            sql.push_str(" AND assignee = ?");
            params.push(SqlParam::String(assignee.clone()));
        }
        if let Some(label) = &filter.label {
            sql.push_str(" AND labels LIKE ?");
            params.push(SqlParam::String(format!("%{}%", label)));
        }

        sql.push_str(" ORDER BY priority ASC, created_at DESC");
        if let Some(limit) = filter.limit {
            sql.push_str(" LIMIT ?");
            params.push(SqlParam::I64(limit as i64));
        }

        let rows = self.executor.query_all(&sql, &params).await?;
        let mut issues = Vec::with_capacity(rows.len());
        for row in rows {
            issues.push(
                row_convert::row_to_project_issue(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode project issue row", e))?,
            );
        }
        Ok(issues)
    }

    // ========== Dependency operations ==========

    async fn add_dependency(&self, dep: &ProjectDependency) -> Result<()> {
        let params = [
            SqlParam::String(dep.id.clone()),
            SqlParam::String(dep.from_issue_id.clone()),
            SqlParam::String(dep.to_issue_id.clone()),
            SqlParam::String(dep.dependency_type.as_str().to_string()),
            SqlParam::I64(dep.created_at),
        ];

        self.executor
            .execute(
                r"
                INSERT INTO project_dependencies (
                    id, from_issue_id, to_issue_id, dependency_type, created_at
                ) VALUES (?, ?, ?, ?, ?)
                ",
                &params,
            )
            .await?;

        debug!("Added dependency: {}", dep.id);
        Ok(())
    }

    async fn remove_dependency(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM project_dependencies WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await?;

        debug!("Removed dependency: {}", id);
        Ok(())
    }

    async fn list_dependencies(&self, project_id: &str) -> Result<Vec<ProjectDependency>> {
        let rows = self
            .executor
            .query_all(
                r"
                SELECT pd.* FROM project_dependencies pd
                JOIN project_issues pi ON pd.from_issue_id = pi.id
                WHERE pi.project_id = ?
                ORDER BY pd.created_at DESC
                ",
                &[SqlParam::String(project_id.to_string())],
            )
            .await?;

        let mut deps = Vec::with_capacity(rows.len());
        for row in rows {
            deps.push(
                row_convert::row_to_project_dependency(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode project dependency row", e))?,
            );
        }
        Ok(deps)
    }

    // ========== Decision operations ==========

    async fn create_decision(&self, decision: &ProjectDecision) -> Result<()> {
        let params = [
            SqlParam::String(decision.id.clone()),
            SqlParam::String(decision.project_id.clone()),
            decision
                .issue_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
            SqlParam::String(decision.title.clone()),
            SqlParam::String(decision.context.clone()),
            SqlParam::String(decision.decision.clone()),
            SqlParam::String(decision.consequences.clone()),
            SqlParam::I64(decision.created_at),
        ];

        self.executor
            .execute(
                r"
                INSERT INTO project_decisions (
                    id, project_id, issue_id, title, context, decision, consequences, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ",
                &params,
            )
            .await?;

        debug!("Created decision: {}", decision.id);
        Ok(())
    }

    async fn get_decision(&self, id: &str) -> Result<Option<ProjectDecision>> {
        let row = self
            .executor
            .query_one(
                "SELECT * FROM project_decisions WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await?;

        match row {
            Some(r) => Ok(Some(
                row_convert::row_to_project_decision(r.as_ref())
                    .map_err(|e| Error::memory_with_source("decode project decision row", e))?,
            )),
            None => Ok(None),
        }
    }

    async fn list_decisions(&self, project_id: &str) -> Result<Vec<ProjectDecision>> {
        let rows = self
            .executor
            .query_all(
                "SELECT * FROM project_decisions WHERE project_id = ? ORDER BY created_at DESC",
                &[SqlParam::String(project_id.to_string())],
            )
            .await?;

        let mut decisions = Vec::with_capacity(rows.len());
        for row in rows {
            decisions.push(
                row_convert::row_to_project_decision(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode project decision row", e))?,
            );
        }
        Ok(decisions)
    }
}
