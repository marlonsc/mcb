use std::path::Path;

use async_trait::async_trait;

use crate::entities::project::{
    DependencyType, IssueFilter, IssueStatus, IssueType, PhaseStatus, Project, ProjectDecision,
    ProjectDependency, ProjectIssue, ProjectPhase, ProjectType,
};
use crate::error::Result;

/// Detects project types inside a repository path
#[async_trait]
pub trait ProjectDetectorService: Send + Sync {
    /// Detect all project types under the given path
    async fn detect_all(&self, path: &Path) -> Vec<ProjectType>;
}

/// Service for managing project workflow resources (phases, issues, dependencies, decisions).
#[async_trait]
pub trait ProjectServiceInterface: Send + Sync {
    // Project operations
    /// Gets a project by ID.
    async fn get_project(&self, id: &str) -> Result<Project>;
    /// Lists all registered projects.
    async fn list_projects(&self) -> Result<Vec<Project>>;

    // Phase operations
    /// Creates a new phase.
    async fn create_phase(
        &self,
        project_id: &str,
        name: String,
        description: String,
    ) -> Result<String>;
    /// Updates an existing phase.
    async fn update_phase(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
        status: Option<PhaseStatus>,
    ) -> Result<()>;
    /// Lists all phases for a project.
    async fn list_phases(&self, project_id: &str) -> Result<Vec<ProjectPhase>>;
    /// Deletes a phase.
    async fn delete_phase(&self, id: &str) -> Result<()>;

    // Issue operations
    /// Creates a new issue.
    async fn create_issue(
        &self,
        project_id: &str,
        title: String,
        description: String,
        issue_type: IssueType,
        priority: i32,
        phase_id: Option<String>,
        assignee: Option<String>,
        labels: Vec<String>,
    ) -> Result<String>;
    /// Updates an existing issue.
    async fn update_issue(
        &self,
        id: &str,
        title: Option<String>,
        description: Option<String>,
        status: Option<IssueStatus>,
        priority: Option<i32>,
        assignee: Option<String>,
        labels: Option<Vec<String>>,
    ) -> Result<()>;
    /// Lists issues for a project with optional filtering.
    async fn list_issues(
        &self,
        project_id: &str,
        filter: Option<IssueFilter>,
    ) -> Result<Vec<ProjectIssue>>;
    /// Deletes an issue.
    async fn delete_issue(&self, id: &str) -> Result<()>;

    // Dependency operations
    /// Adds a dependency between issues.
    async fn add_dependency(
        &self,
        from_issue_id: String,
        to_issue_id: String,
        dependency_type: DependencyType,
    ) -> Result<String>;
    /// Removes a dependency.
    async fn remove_dependency(&self, id: &str) -> Result<()>;
    /// Lists dependencies for a project.
    async fn list_dependencies(&self, project_id: &str) -> Result<Vec<ProjectDependency>>;

    // Decision operations
    /// Records a new decision.
    async fn create_decision(
        &self,
        project_id: &str,
        title: String,
        context: String,
        decision: String,
        consequences: String,
        issue_id: Option<String>,
    ) -> Result<String>;
    /// Lists decisions for a project.
    async fn list_decisions(&self, project_id: &str) -> Result<Vec<ProjectDecision>>;
    /// Deletes a decision.
    async fn delete_decision(&self, id: &str) -> Result<()>;
}
