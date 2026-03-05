//! VCS entity repository implementation.
//!
//! Implements `VcsRepositoryRegistry`, `VcsBranchRegistry`, `VcsWorktreeRegistry`,
//! and `AgentAssignmentManager` for managing repositories, branches, worktrees,
//! and agent worktree assignments.

use super::*;

sea_impl_crud_scoped!(VcsRepositoryRegistry for SeaOrmEntityRepository { db: db,
    entity: repository, domain: Repository, label: "Repository",
    scope_col: repository::Column::OrgId,
    create: create_repository(repo),
    get: get_repository,
    list: list_repositories(repository::Column::ProjectId => project_id),
    update: update_repository(repo),
    delete: delete_repository
});

sea_impl_crud_mixed!(VcsBranchRegistry for SeaOrmEntityRepository { db: db,
    entity: branch, domain: Branch, label: "Branch",
    scope_col: branch::Column::OrgId,
    create: create_branch(b),
    get: get_branch,
    list: list_branches(branch::Column::RepositoryId => repository_id),
    update: update_branch(b),
    delete: delete_branch(id),
});

sea_impl_crud!(VcsWorktreeRegistry for SeaOrmEntityRepository { db: db,
    entity: worktree, domain: Worktree, label: "Worktree",
    create: create_worktree(wt),
    get: get_worktree(id),
    list: list_worktrees(worktree::Column::RepositoryId => repository_id),
    update: update_worktree(wt),
    delete: delete_worktree(id)
});

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
        sea_repo_list!(self.db(), agent_worktree_assignment, AgentWorktreeAssignment, "list assignments",
            agent_worktree_assignment::Column::WorktreeId => worktree_id)
    }

    async fn release_assignment(&self, id: &str, released_at: i64) -> Result<()> {
        sea_repo_set_field!(
            self.db(),
            agent_worktree_assignment,
            id,
            "Assignment",
            "release assignment",
            released_at = Some(released_at)
        )
    }
}
