use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::repository::{Branch, Repository};
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree};
use mcb_domain::error::Result;
use mcb_domain::ports::repositories::VcsEntityRepository;
use mcb_domain::ports::services::VcsEntityServiceInterface;

/// Application-layer service for VCS entity CRUD operations.
pub struct VcsEntityServiceImpl {
    repository: Arc<dyn VcsEntityRepository>,
}

impl VcsEntityServiceImpl {
    /// Create a new [`VcsEntityServiceImpl`] backed by the given repository.
    pub fn new(repository: Arc<dyn VcsEntityRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl VcsEntityServiceInterface for VcsEntityServiceImpl {
    // -- Repository --

    async fn create_repository(&self, repo: &Repository) -> Result<()> {
        self.repository.create_repository(repo).await
    }

    async fn get_repository(&self, org_id: &str, id: &str) -> Result<Repository> {
        self.repository.get_repository(org_id, id).await
    }

    async fn list_repositories(&self, org_id: &str, project_id: &str) -> Result<Vec<Repository>> {
        self.repository.list_repositories(org_id, project_id).await
    }

    async fn update_repository(&self, repo: &Repository) -> Result<()> {
        self.repository.update_repository(repo).await
    }

    async fn delete_repository(&self, org_id: &str, id: &str) -> Result<()> {
        self.repository.delete_repository(org_id, id).await
    }

    // -- Branch --

    async fn create_branch(&self, branch: &Branch) -> Result<()> {
        self.repository.create_branch(branch).await
    }

    async fn get_branch(&self, id: &str) -> Result<Branch> {
        self.repository.get_branch(id).await
    }

    async fn list_branches(&self, repository_id: &str) -> Result<Vec<Branch>> {
        self.repository.list_branches(repository_id).await
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        self.repository.update_branch(branch).await
    }

    async fn delete_branch(&self, id: &str) -> Result<()> {
        self.repository.delete_branch(id).await
    }

    // -- Worktree --

    async fn create_worktree(&self, wt: &Worktree) -> Result<()> {
        self.repository.create_worktree(wt).await
    }

    async fn get_worktree(&self, id: &str) -> Result<Worktree> {
        self.repository.get_worktree(id).await
    }

    async fn list_worktrees(&self, repository_id: &str) -> Result<Vec<Worktree>> {
        self.repository.list_worktrees(repository_id).await
    }

    async fn update_worktree(&self, wt: &Worktree) -> Result<()> {
        self.repository.update_worktree(wt).await
    }

    async fn delete_worktree(&self, id: &str) -> Result<()> {
        self.repository.delete_worktree(id).await
    }

    // -- Assignment --

    async fn create_assignment(&self, asgn: &AgentWorktreeAssignment) -> Result<()> {
        self.repository.create_assignment(asgn).await
    }

    async fn get_assignment(&self, id: &str) -> Result<AgentWorktreeAssignment> {
        self.repository.get_assignment(id).await
    }

    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>> {
        self.repository
            .list_assignments_by_worktree(worktree_id)
            .await
    }

    async fn release_assignment(&self, id: &str, released_at: i64) -> Result<()> {
        self.repository.release_assignment(id, released_at).await
    }
}
