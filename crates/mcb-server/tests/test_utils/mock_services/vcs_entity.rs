use async_trait::async_trait;
use mcb_domain::entities::repository::{Branch, Repository};
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::repositories::VcsEntityRepository;

#[allow(dead_code)]
pub struct MockVcsEntityService;

#[allow(dead_code)]
impl MockVcsEntityService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockVcsEntityService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VcsEntityRepository for MockVcsEntityService {
    async fn create_repository(&self, _repo: &Repository) -> Result<()> {
        Ok(())
    }
    async fn get_repository(&self, _org_id: &str, _id: &str) -> Result<Repository> {
        Err(Error::not_found("not found"))
    }
    async fn list_repositories(&self, _org_id: &str, _project_id: &str) -> Result<Vec<Repository>> {
        Ok(vec![])
    }
    async fn update_repository(&self, _repo: &Repository) -> Result<()> {
        Ok(())
    }
    async fn delete_repository(&self, _org_id: &str, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn create_branch(&self, _branch: &Branch) -> Result<()> {
        Ok(())
    }
    async fn get_branch(&self, _id: &str) -> Result<Branch> {
        Err(Error::not_found("not found"))
    }
    async fn list_branches(&self, _repository_id: &str) -> Result<Vec<Branch>> {
        Ok(vec![])
    }
    async fn update_branch(&self, _branch: &Branch) -> Result<()> {
        Ok(())
    }
    async fn delete_branch(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn create_worktree(&self, _wt: &Worktree) -> Result<()> {
        Ok(())
    }
    async fn get_worktree(&self, _id: &str) -> Result<Worktree> {
        Err(Error::not_found("not found"))
    }
    async fn list_worktrees(&self, _repository_id: &str) -> Result<Vec<Worktree>> {
        Ok(vec![])
    }
    async fn update_worktree(&self, _wt: &Worktree) -> Result<()> {
        Ok(())
    }
    async fn delete_worktree(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn create_assignment(&self, _asgn: &AgentWorktreeAssignment) -> Result<()> {
        Ok(())
    }
    async fn get_assignment(&self, _id: &str) -> Result<AgentWorktreeAssignment> {
        Err(Error::not_found("not found"))
    }
    async fn list_assignments_by_worktree(
        &self,
        _worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>> {
        Ok(vec![])
    }
    async fn release_assignment(&self, _id: &str, _released_at: i64) -> Result<()> {
        Ok(())
    }
}
