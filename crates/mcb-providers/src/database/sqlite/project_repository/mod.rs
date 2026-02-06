//! SQLite project repository using the domain port [`DatabaseExecutor`].
//!
//! Implements [`ProjectRepository`] via [`DatabaseExecutor`]; no direct sqlx in this module.

mod decision;
mod dependency;
mod issue;
mod phase;
mod project;

use async_trait::async_trait;
use mcb_domain::entities::project::{
    Project, ProjectDecision, ProjectDependency, ProjectIssue, ProjectPhase,
};
use mcb_domain::error::Result;
use mcb_domain::ports::infrastructure::database::DatabaseExecutor;
use mcb_domain::ports::repositories::{IssueFilter, ProjectRepository};
use std::sync::Arc;

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
        project::create(&self.executor, project).await
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Project>> {
        project::get_by_id(&self.executor, id).await
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Project>> {
        project::get_by_name(&self.executor, name).await
    }

    async fn get_by_path(&self, path: &str) -> Result<Option<Project>> {
        project::get_by_path(&self.executor, path).await
    }

    async fn list(&self) -> Result<Vec<Project>> {
        project::list(&self.executor).await
    }

    async fn update(&self, project: &Project) -> Result<()> {
        project::update(&self.executor, project).await
    }

    async fn delete(&self, id: &str) -> Result<()> {
        project::delete(&self.executor, id).await
    }

    // ========== Phase operations ==========

    async fn create_phase(&self, phase: &ProjectPhase) -> Result<()> {
        phase::create_phase(&self.executor, phase).await
    }

    async fn get_phase(&self, id: &str) -> Result<Option<ProjectPhase>> {
        phase::get_phase(&self.executor, id).await
    }

    async fn update_phase(&self, phase: &ProjectPhase) -> Result<()> {
        phase::update_phase(&self.executor, phase).await
    }

    async fn list_phases(&self, project_id: &str) -> Result<Vec<ProjectPhase>> {
        phase::list_phases(&self.executor, project_id).await
    }

    // ========== Issue operations ==========

    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()> {
        issue::create_issue(&self.executor, issue).await
    }

    async fn get_issue(&self, id: &str) -> Result<Option<ProjectIssue>> {
        issue::get_issue(&self.executor, id).await
    }

    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()> {
        issue::update_issue(&self.executor, issue).await
    }

    async fn list_issues(&self, project_id: &str) -> Result<Vec<ProjectIssue>> {
        issue::list_issues(&self.executor, project_id).await
    }

    async fn filter_issues(&self, filter: &IssueFilter) -> Result<Vec<ProjectIssue>> {
        issue::filter_issues(&self.executor, filter).await
    }

    // ========== Dependency operations ==========

    async fn add_dependency(&self, dep: &ProjectDependency) -> Result<()> {
        dependency::add_dependency(&self.executor, dep).await
    }

    async fn remove_dependency(&self, id: &str) -> Result<()> {
        dependency::remove_dependency(&self.executor, id).await
    }

    async fn list_dependencies(&self, project_id: &str) -> Result<Vec<ProjectDependency>> {
        dependency::list_dependencies(&self.executor, project_id).await
    }

    // ========== Decision operations ==========

    async fn create_decision(&self, decision: &ProjectDecision) -> Result<()> {
        decision::create_decision(&self.executor, decision).await
    }

    async fn get_decision(&self, id: &str) -> Result<Option<ProjectDecision>> {
        decision::get_decision(&self.executor, id).await
    }

    async fn list_decisions(&self, project_id: &str) -> Result<Vec<ProjectDecision>> {
        decision::list_decisions(&self.executor, project_id).await
    }
}
