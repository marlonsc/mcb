use crate::entities::project::{
    IssueStatus, IssueType, Project, ProjectDecision, ProjectDependency, ProjectIssue, ProjectPhase,
};
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Filter for querying project issues with optional constraints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueFilter {
    pub project_id: Option<String>,
    pub phase_id: Option<String>,
    pub issue_type: Option<IssueType>,
    pub status: Option<IssueStatus>,
    pub priority: Option<i32>,
    pub assignee: Option<String>,
    pub label: Option<String>,
    pub limit: Option<usize>,
}

/// Port for project persistence (CRUD operations on Project entities and related workflow data).
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    // Project CRUD
    async fn create(&self, project: &Project) -> Result<()>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Project>>;
    async fn get_by_name(&self, name: &str) -> Result<Option<Project>>;
    async fn get_by_path(&self, path: &str) -> Result<Option<Project>>;
    async fn list(&self) -> Result<Vec<Project>>;
    async fn update(&self, project: &Project) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;

    // Phase operations
    async fn create_phase(&self, phase: &ProjectPhase) -> Result<()>;
    async fn get_phase(&self, id: &str) -> Result<Option<ProjectPhase>>;
    async fn update_phase(&self, phase: &ProjectPhase) -> Result<()>;
    async fn list_phases(&self, project_id: &str) -> Result<Vec<ProjectPhase>>;

    // Issue operations
    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()>;
    async fn get_issue(&self, id: &str) -> Result<Option<ProjectIssue>>;
    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()>;
    async fn list_issues(&self, project_id: &str) -> Result<Vec<ProjectIssue>>;
    async fn filter_issues(&self, filter: &IssueFilter) -> Result<Vec<ProjectIssue>>;

    // Dependency operations
    async fn add_dependency(&self, dep: &ProjectDependency) -> Result<()>;
    async fn remove_dependency(&self, id: &str) -> Result<()>;
    async fn list_dependencies(&self, project_id: &str) -> Result<Vec<ProjectDependency>>;

    // Decision operations
    async fn create_decision(&self, decision: &ProjectDecision) -> Result<()>;
    async fn get_decision(&self, id: &str) -> Result<Option<ProjectDecision>>;
    async fn list_decisions(&self, project_id: &str) -> Result<Vec<ProjectDecision>>;
}
