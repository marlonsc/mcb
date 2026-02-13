//! VCS Entity Repository Port
//!
//! # Overview
//! Defines the interface for persisting VCS-related entities including repositories,
//! branches, worktrees, and agent assignments.
use async_trait::async_trait;

use crate::entities::repository::{Branch, Repository};
use crate::entities::worktree::{AgentWorktreeAssignment, Worktree};
use crate::error::Result;

#[async_trait]
/// Defines behavior for VcsEntityRepository.
// TODO(architecture): Consider splitting into smaller interfaces (ISP).
// Current interface combines Repository, Branch, Worktree, and Assignment management.
// TODO(PORT003): Port VcsEntityRepository has 19 methods (>10) - Consider splitting into smaller interfaces (ISP)
pub trait VcsEntityRepository: Send + Sync {
    // -- Repository CRUD --
    /// Performs the create repository operation.
    async fn create_repository(&self, repo: &Repository) -> Result<()>;
    /// Performs the get repository operation.
    async fn get_repository(&self, org_id: &str, id: &str) -> Result<Repository>;
    /// Performs the list repositories operation.
    async fn list_repositories(&self, org_id: &str, project_id: &str) -> Result<Vec<Repository>>;
    /// Performs the update repository operation.
    async fn update_repository(&self, repo: &Repository) -> Result<()>;
    /// Performs the delete repository operation.
    async fn delete_repository(&self, org_id: &str, id: &str) -> Result<()>;

    // -- Branch CRUD --
    /// Performs the create branch operation.
    async fn create_branch(&self, branch: &Branch) -> Result<()>;
    /// Performs the get branch operation.
    async fn get_branch(&self, id: &str) -> Result<Branch>;
    /// Performs the list branches operation.
    async fn list_branches(&self, repository_id: &str) -> Result<Vec<Branch>>;
    /// Performs the update branch operation.
    async fn update_branch(&self, branch: &Branch) -> Result<()>;
    /// Performs the delete branch operation.
    async fn delete_branch(&self, id: &str) -> Result<()>;

    // -- Worktree CRUD --
    /// Performs the create worktree operation.
    async fn create_worktree(&self, wt: &Worktree) -> Result<()>;
    /// Performs the get worktree operation.
    async fn get_worktree(&self, id: &str) -> Result<Worktree>;
    /// Performs the list worktrees operation.
    async fn list_worktrees(&self, repository_id: &str) -> Result<Vec<Worktree>>;
    /// Performs the update worktree operation.
    async fn update_worktree(&self, wt: &Worktree) -> Result<()>;
    /// Performs the delete worktree operation.
    async fn delete_worktree(&self, id: &str) -> Result<()>;

    // -- Assignment CRUD --
    /// Performs the create assignment operation.
    async fn create_assignment(&self, asgn: &AgentWorktreeAssignment) -> Result<()>;
    /// Performs the get assignment operation.
    async fn get_assignment(&self, id: &str) -> Result<AgentWorktreeAssignment>;
    /// Performs the list assignments by worktree operation.
    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>>;
    /// Performs the release assignment operation.
    async fn release_assignment(&self, id: &str, released_at: i64) -> Result<()>;
}
