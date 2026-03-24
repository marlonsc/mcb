//! VCS repository ports.

use async_trait::async_trait;

use crate::entities::repository::{Branch, Repository};
use crate::entities::worktree::{AgentWorktreeAssignment, Worktree};
use crate::error::Result;

define_crud_port! {
    /// Registry for VCS repositories.
    pub trait VcsRepositoryRegistry {
        entity: Repository,
        create: create_repository,
        get: get_repository(org_id, id),
        list: list_repositories(org_id, project_id),
        update: update_repository,
        delete: delete_repository(org_id, id),
    }
}

/// Registry for VCS branches.
///
/// Mixed scope: get/list are org-scoped but delete takes only id.
#[async_trait]
pub trait VcsBranchRegistry: Send + Sync {
    /// Create a branch.
    async fn create_branch(&self, branch: &Branch) -> Result<()>;
    /// Get a branch by ID.
    async fn get_branch(&self, org_id: &str, id: &str) -> Result<Branch>;
    /// List branches for a repository.
    async fn list_branches(&self, org_id: &str, repository_id: &str) -> Result<Vec<Branch>>;
    /// Update a branch.
    async fn update_branch(&self, branch: &Branch) -> Result<()>;
    /// Delete a branch.
    async fn delete_branch(&self, id: &str) -> Result<()>;
}

define_crud_port! {
    /// Registry for VCS worktrees.
    pub trait VcsWorktreeRegistry {
        entity: Worktree,
        create: create_worktree,
        get: get_worktree(id),
        list: list_worktrees(repository_id),
        update: update_worktree,
        delete: delete_worktree(id),
    }
}

/// Manager for agent-worktree assignments.
#[async_trait]
pub trait AgentAssignmentManager: Send + Sync {
    /// Create an agent-worktree assignment.
    async fn create_assignment(&self, asgn: &AgentWorktreeAssignment) -> Result<()>;
    /// Get an assignment by ID.
    async fn get_assignment(&self, id: &str) -> Result<AgentWorktreeAssignment>;
    /// List assignments for a worktree.
    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>>;
    /// Release an assignment.
    async fn release_assignment(&self, id: &str, released_at: i64) -> Result<()>;
}

define_aggregate! {
    /// Aggregate trait for VCS entity management.
    #[async_trait]
    pub trait VcsEntityRepository = VcsRepositoryRegistry + VcsBranchRegistry + VcsWorktreeRegistry + AgentAssignmentManager;
}
