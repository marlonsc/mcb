use async_trait::async_trait;

pub use crate::entities::project::{
    IssueFilter, Project, ProjectDecision, ProjectDependency, ProjectIssue, ProjectPhase,
};
use crate::error::Result;

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
    /// Deletes a phase by its identifier.
    async fn delete_phase(&self, id: &str) -> Result<()>;

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
    /// Deletes an issue by its identifier.
    async fn delete_issue(&self, id: &str) -> Result<()>;

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
    /// Deletes a decision by its identifier.
    async fn delete_decision(&self, id: &str) -> Result<()>;
}
