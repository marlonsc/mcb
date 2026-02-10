use async_trait::async_trait;

use crate::entities::repository::{Branch, Repository};
use crate::entities::worktree::{AgentWorktreeAssignment, Worktree};
use crate::error::Result;

#[async_trait]
pub trait VcsEntityRepository: Send + Sync {
    // -- Repository CRUD --
    async fn create_repository(&self, repo: &Repository) -> Result<()>;
    async fn get_repository(&self, org_id: &str, id: &str) -> Result<Option<Repository>>;
    async fn list_repositories(&self, org_id: &str, project_id: &str) -> Result<Vec<Repository>>;
    async fn update_repository(&self, repo: &Repository) -> Result<()>;
    async fn delete_repository(&self, org_id: &str, id: &str) -> Result<()>;

    // -- Branch CRUD --
    async fn create_branch(&self, branch: &Branch) -> Result<()>;
    async fn get_branch(&self, id: &str) -> Result<Option<Branch>>;
    async fn list_branches(&self, repository_id: &str) -> Result<Vec<Branch>>;
    async fn update_branch(&self, branch: &Branch) -> Result<()>;
    async fn delete_branch(&self, id: &str) -> Result<()>;

    // -- Worktree CRUD --
    async fn create_worktree(&self, wt: &Worktree) -> Result<()>;
    async fn get_worktree(&self, id: &str) -> Result<Option<Worktree>>;
    async fn list_worktrees(&self, repository_id: &str) -> Result<Vec<Worktree>>;
    async fn update_worktree(&self, wt: &Worktree) -> Result<()>;
    async fn delete_worktree(&self, id: &str) -> Result<()>;

    // -- Assignment CRUD --
    async fn create_assignment(&self, asgn: &AgentWorktreeAssignment) -> Result<()>;
    async fn get_assignment(&self, id: &str) -> Result<Option<AgentWorktreeAssignment>>;
    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>>;
    async fn release_assignment(&self, id: &str, released_at: i64) -> Result<()>;
}
