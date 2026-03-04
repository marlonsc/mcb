//! VCS entity repository implementation.
//!
//! Implements `VcsEntityRepository` for managing repositories, branches, worktrees,
//! and agent worktree assignments.

use super::*;

#[async_trait]
impl VcsRepositoryRegistry for SeaOrmEntityRepository {
    async fn create_repository(&self, repo: &Repository) -> Result<()> {
        sea_repo_insert!(self.db(), repository, repo, "create repository")
    }

    async fn get_repository(&self, org_id: &str, id: &str) -> Result<Repository> {
        sea_repo_get_filtered!(self.db(), repository, Repository, "Repository", id, "get repository", repository::Column::OrgId => org_id)
    }

    async fn list_repositories(&self, org_id: &str, project_id: &str) -> Result<Vec<Repository>> {
        sea_repo_list!(self.db(), repository, Repository, "list repositories", repository::Column::OrgId => org_id, repository::Column::ProjectId => project_id)
    }

    async fn update_repository(&self, repo: &Repository) -> Result<()> {
        sea_repo_update!(self.db(), repository, repo, "update repository")
    }

    async fn delete_repository(&self, org_id: &str, id: &str) -> Result<()> {
        sea_repo_delete_filtered!(self.db(), repository, id, "delete repository", repository::Column::OrgId => org_id)
    }
}

#[async_trait]
impl VcsBranchRegistry for SeaOrmEntityRepository {
    async fn create_branch(&self, branch: &Branch) -> Result<()> {
        sea_repo_insert!(self.db(), branch, branch, "create branch")
    }

    async fn get_branch(&self, org_id: &str, id: &str) -> Result<Branch> {
        sea_repo_get_filtered!(self.db(), branch, Branch, "Branch", id, "get branch", branch::Column::OrgId => org_id)
    }

    async fn list_branches(&self, org_id: &str, repository_id: &str) -> Result<Vec<Branch>> {
        sea_repo_list!(self.db(), branch, Branch, "list branches", branch::Column::OrgId => org_id, branch::Column::RepositoryId => repository_id)
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        sea_repo_update!(self.db(), branch, branch, "update branch")
    }

    async fn delete_branch(&self, id: &str) -> Result<()> {
        sea_repo_delete!(self.db(), branch, id, "delete branch")
    }
}

#[async_trait]
impl VcsWorktreeRegistry for SeaOrmEntityRepository {
    async fn create_worktree(&self, wt: &Worktree) -> Result<()> {
        sea_repo_insert!(self.db(), worktree, wt, "create worktree")
    }

    async fn get_worktree(&self, id: &str) -> Result<Worktree> {
        sea_repo_get!(
            self.db(),
            worktree,
            Worktree,
            "Worktree",
            id,
            "get worktree"
        )
    }

    async fn list_worktrees(&self, repository_id: &str) -> Result<Vec<Worktree>> {
        sea_repo_list!(self.db(), worktree, Worktree, "list worktrees", worktree::Column::RepositoryId => repository_id)
    }

    async fn update_worktree(&self, wt: &Worktree) -> Result<()> {
        sea_repo_update!(self.db(), worktree, wt, "update worktree")
    }

    async fn delete_worktree(&self, id: &str) -> Result<()> {
        sea_repo_delete!(self.db(), worktree, id, "delete worktree")
    }
}

#[async_trait]
impl AgentAssignmentManager for SeaOrmEntityRepository {
    async fn create_assignment(&self, asgn: &AgentWorktreeAssignment) -> Result<()> {
        sea_repo_insert!(
            self.db(),
            agent_worktree_assignment,
            asgn,
            "create assignment"
        )
    }

    async fn get_assignment(&self, id: &str) -> Result<AgentWorktreeAssignment> {
        sea_repo_get!(
            self.db(),
            agent_worktree_assignment,
            AgentWorktreeAssignment,
            "Assignment",
            id,
            "get assignment"
        )
    }

    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>> {
        sea_repo_list!(self.db(), agent_worktree_assignment, AgentWorktreeAssignment, "list assignments", agent_worktree_assignment::Column::WorktreeId => worktree_id)
    }

    async fn release_assignment(&self, id: &str, released_at: i64) -> Result<()> {
        use sea_orm::ActiveValue;

        let model = agent_worktree_assignment::Entity::find_by_id(id)
            .one(self.db())
            .await
            .map_err(crate::database::seaorm::repos::common::db_error(
                "release assignment",
            ))?;
        let m = Error::not_found_or(model, "Assignment", id)?;

        let mut active: agent_worktree_assignment::ActiveModel = m.into();
        active.released_at = ActiveValue::Set(Some(released_at));
        active.update(self.db()).await.map_err(
            crate::database::seaorm::repos::common::db_error("release assignment"),
        )?;
        Ok(())
    }
}
