//! Project repository ports.

use async_trait::async_trait;

use crate::entities::project::{
    IssueFilter, Project, ProjectDecision, ProjectDependency, ProjectIssue, ProjectPhase,
};
use crate::error::Result;

/// Port for project persistence with row-level tenant isolation.
///
/// Covers the full project management domain: projects, phases, issues,
/// dependencies, and decisions.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    // ── Project ──────────────────────────────────────────────────────────

    /// Create a project.
    async fn create(&self, project: &Project) -> Result<()>;
    /// Get a project by ID.
    async fn get_by_id(&self, org_id: &str, id: &str) -> Result<Project>;
    /// Get a project by name.
    async fn get_by_name(&self, org_id: &str, name: &str) -> Result<Project>;
    /// Get a project by path.
    async fn get_by_path(&self, org_id: &str, path: &str) -> Result<Project>;
    /// List projects in an organization.
    async fn list(&self, org_id: &str) -> Result<Vec<Project>>;
    /// Update a project.
    async fn update(&self, project: &Project) -> Result<()>;
    /// Delete a project.
    async fn delete(&self, org_id: &str, id: &str) -> Result<()>;

    // ── Phase ────────────────────────────────────────────────────────────

    /// Create a project phase.
    async fn create_phase(&self, phase: &ProjectPhase) -> Result<()>;
    /// Get a phase by ID.
    async fn get_phase(&self, id: &str) -> Result<ProjectPhase>;
    /// List phases for a project (ordered by sequence).
    async fn list_phases(&self, project_id: &str) -> Result<Vec<ProjectPhase>>;
    /// Update a phase.
    async fn update_phase(&self, phase: &ProjectPhase) -> Result<()>;
    /// Delete a phase.
    async fn delete_phase(&self, id: &str) -> Result<()>;

    // ── Issue ────────────────────────────────────────────────────────────

    /// Create a project issue.
    async fn create_issue(&self, issue: &ProjectIssue) -> Result<()>;
    /// Get an issue by ID (org-scoped).
    async fn get_issue(&self, org_id: &str, id: &str) -> Result<ProjectIssue>;
    /// List issues for a project (org-scoped).
    async fn list_issues(&self, org_id: &str, project_id: &str) -> Result<Vec<ProjectIssue>>;
    /// List issues using a rich filter.
    async fn list_issues_filtered(
        &self,
        org_id: &str,
        filter: &IssueFilter,
    ) -> Result<Vec<ProjectIssue>>;
    /// Update an issue.
    async fn update_issue(&self, issue: &ProjectIssue) -> Result<()>;
    /// Delete an issue (org-scoped).
    async fn delete_issue(&self, org_id: &str, id: &str) -> Result<()>;

    // ── Dependency ───────────────────────────────────────────────────────

    /// Create a dependency edge between issues.
    async fn create_dependency(&self, dependency: &ProjectDependency) -> Result<()>;
    /// List dependencies for an issue (both directions).
    async fn list_dependencies(&self, issue_id: &str) -> Result<Vec<ProjectDependency>>;
    /// Delete a dependency edge.
    async fn delete_dependency(&self, id: &str) -> Result<()>;

    // ── Decision ─────────────────────────────────────────────────────────

    /// Create a project decision.
    async fn create_decision(&self, decision: &ProjectDecision) -> Result<()>;
    /// Get a decision by ID.
    async fn get_decision(&self, id: &str) -> Result<ProjectDecision>;
    /// List decisions for a project.
    async fn list_decisions(&self, project_id: &str) -> Result<Vec<ProjectDecision>>;
    /// Update a decision.
    async fn update_decision(&self, decision: &ProjectDecision) -> Result<()>;
    /// Delete a decision.
    async fn delete_decision(&self, id: &str) -> Result<()>;
}
