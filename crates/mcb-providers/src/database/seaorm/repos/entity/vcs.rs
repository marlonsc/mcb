//! VCS entity repository implementation.
//!
//! Implements `VcsEntityRepository` for managing repositories, branches, worktrees,
//! and agent worktree assignments.

use super::*;

#[async_trait]
impl VcsEntityRepository for SeaOrmEntityRepository {
    // -- Repository --

    async fn create_repository(&self, repo: &Repository) -> Result<()> {
        sea_insert!(self, repository, repo)
    }

    async fn get_repository(&self, org_id: &str, id: &str) -> Result<Repository> {
        sea_get_filtered!(self, repository, Repository, "Repository", id, repository::Column::OrgId => org_id)
    }

    async fn list_repositories(&self, org_id: &str, project_id: &str) -> Result<Vec<Repository>> {
        sea_list!(self, repository, Repository, repository::Column::OrgId => org_id, repository::Column::ProjectId => project_id)
    }

    async fn update_repository(&self, repo: &Repository) -> Result<()> {
        sea_update!(self, repository, repo)
    }

    async fn delete_repository(&self, org_id: &str, id: &str) -> Result<()> {
        sea_delete_filtered!(self, repository, id, repository::Column::OrgId => org_id)
    }

    // -- Branch --

    async fn create_branch(&self, branch: &Branch) -> Result<()> {
        sea_insert!(self, branch, branch)
    }

    async fn get_branch(&self, org_id: &str, id: &str) -> Result<Branch> {
        sea_get_filtered!(self, branch, Branch, "Branch", id, branch::Column::OrgId => org_id)
    }

    async fn list_branches(&self, org_id: &str, repository_id: &str) -> Result<Vec<Branch>> {
        sea_list!(self, branch, Branch, branch::Column::OrgId => org_id, branch::Column::RepositoryId => repository_id)
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        sea_update!(self, branch, branch)
    }

    async fn delete_branch(&self, id: &str) -> Result<()> {
        sea_delete!(self, branch, id)
    }

    // -- Worktree --

    async fn create_worktree(&self, wt: &Worktree) -> Result<()> {
        sea_insert!(self, worktree, wt)
    }

    async fn get_worktree(&self, id: &str) -> Result<Worktree> {
        sea_get!(self, worktree, Worktree, "Worktree", id)
    }

    async fn list_worktrees(&self, repository_id: &str) -> Result<Vec<Worktree>> {
        sea_list!(self, worktree, Worktree, worktree::Column::RepositoryId => repository_id)
    }

    async fn update_worktree(&self, wt: &Worktree) -> Result<()> {
        sea_update!(self, worktree, wt)
    }

    async fn delete_worktree(&self, id: &str) -> Result<()> {
        sea_delete!(self, worktree, id)
    }

    // -- Assignment --

    async fn create_assignment(&self, asgn: &AgentWorktreeAssignment) -> Result<()> {
        sea_insert!(self, agent_worktree_assignment, asgn)
    }

    async fn get_assignment(&self, id: &str) -> Result<AgentWorktreeAssignment> {
        sea_get!(
            self,
            agent_worktree_assignment,
            AgentWorktreeAssignment,
            "Assignment",
            id
        )
    }

    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>> {
        sea_list!(self, agent_worktree_assignment, AgentWorktreeAssignment, agent_worktree_assignment::Column::WorktreeId => worktree_id)
    }

    async fn release_assignment(&self, id: &str, released_at: i64) -> Result<()> {
        use sea_orm::ActiveValue;

        let model = agent_worktree_assignment::Entity::find_by_id(id)
            .one(self.db())
            .await
            .map_err(db_err)?;
        let m = Error::not_found_or(model, "Assignment", id)?;

        let mut active: agent_worktree_assignment::ActiveModel = m.into();
        active.released_at = ActiveValue::Set(Some(released_at));
        active.update(self.db()).await.map_err(db_err)?;
        Ok(())
    }
}
