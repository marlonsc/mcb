use crate::entities::project::{
    IssueStatus, IssueType, Project, ProjectDecision, ProjectDependency, ProjectIssue, ProjectPhase,
};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Filter for querying project issues with optional constraints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueFilter {
    /// Filter by project identifier.
    pub project_id: Option<String>,
    /// Filter by project phase identifier.
    pub phase_id: Option<String>,
    /// Filter by type of issue (e.g., bug, task).
    pub issue_type: Option<IssueType>,
    /// Filter by issue status (e.g., open, closed).
    pub status: Option<IssueStatus>,
    /// Filter by priority level.
    pub priority: Option<i32>,
    /// Filter by assigned user.
    pub assignee: Option<String>,
    /// Filter by applied label/tag.
    pub label: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
}

/// Port for project persistence (CRUD operations on Project entities and related workflow data).
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    // Project CRUD
    /// Creates a new project record.
    async fn create(&self, project: &Project) -> Result<()>;
    /// Retrieves a project by its unique identifier.
    async fn get_by_id(&self, id: &str) -> Result<Option<Project>>;
    /// Retrieves a project by its name.
    async fn get_by_name(&self, name: &str) -> Result<Option<Project>>;
    /// Retrieves a project by its filesystem path.
    async fn get_by_path(&self, path: &str) -> Result<Option<Project>>;
    /// Lists all registered projects.
    async fn list(&self) -> Result<Vec<Project>>;
    /// Updates an existing project record.
    async fn update(&self, project: &Project) -> Result<()>;
    /// Deletes a project by its identifier.
    async fn delete(&self, id: &str) -> Result<()>;

    // Phase operations
    /// Creates a new phase within a project.
    async fn create_phase(&self, phase: &ProjectPhase) -> Result<()>;
    /// Retrieves a phase by its unique identifier.
    async fn get_phase(&self, id: &str) -> Result<Option<ProjectPhase>>;
    /// Updates an existing project phase.
    async fn update_phase(&self, phase: &ProjectPhase) -> Result<()>;
    /// Lists all phases associated with a project.
    async fn list_phases(&self, project_id: &str) -> Result<Vec<ProjectPhase>>;

    // Issue operations
    /// Creates a new issue/task.
    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()>;
    /// Retrieves an issue by its unique identifier.
    async fn get_issue(&self, id: &str) -> Result<Option<ProjectIssue>>;
    /// Updates an existing issue record.
    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()>;
    /// Lists all issues associated with a project.
    async fn list_issues(&self, project_id: &str) -> Result<Vec<ProjectIssue>>;
    /// Retrieves a list of issues matching the specified filter criteria.
    async fn filter_issues(&self, filter: &IssueFilter) -> Result<Vec<ProjectIssue>>;

    // Dependency operations
    /// Creates a dependency relationship between two issues.
    async fn add_dependency(&self, dep: &ProjectDependency) -> Result<()>;
    /// Removes a dependency relationship by its identifier.
    async fn remove_dependency(&self, id: &str) -> Result<()>;
    /// Lists all dependencies for issues within a project.
    async fn list_dependencies(&self, project_id: &str) -> Result<Vec<ProjectDependency>>;

    // Decision operations
    /// Records a new project decision.
    async fn create_decision(&self, decision: &ProjectDecision) -> Result<()>;
    /// Retrieves a decision by its unique identifier.
    async fn get_decision(&self, id: &str) -> Result<Option<ProjectDecision>>;
    /// Lists all decisions recorded for a project.
    async fn list_decisions(&self, project_id: &str) -> Result<Vec<ProjectDecision>>;
}
