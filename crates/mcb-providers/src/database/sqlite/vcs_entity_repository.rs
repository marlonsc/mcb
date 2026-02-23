//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md#database)
//!
//! `SQLite` VCS entity repository.

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::repository::{Branch, Repository};
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::VcsEntityRepository;
use mcb_domain::ports::{DatabaseExecutor, SqlParam};
use serde_json::json;

use crate::database::sqlite::row_convert::FromRow;
use crate::utils::sqlite::query as query_helpers;

/// SQLite-backed repository for VCS repositories, branches, worktrees, and assignments.
pub struct SqliteVcsEntityRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

impl SqliteVcsEntityRepository {
    /// Creates a new repository using the provided database executor.
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }
}

#[async_trait]
/// Repository for VCS entities.
impl VcsEntityRepository for SqliteVcsEntityRepository {
    // -- Repository --

    /// Creates a new repository.
    async fn create_repository(&self, repo: &Repository) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO repositories (id, org_id, project_id, name, url, local_path, vcs_type, origin_context, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(repo.id.clone()),
                    SqlParam::String(repo.org_id.clone()),
                    SqlParam::String(repo.project_id.clone()),
                    SqlParam::String(repo.name.clone()),
                    SqlParam::String(repo.url.clone()),
                    SqlParam::String(repo.local_path.clone()),
                    SqlParam::String(repo.vcs_type.to_string()),
                    SqlParam::String(
                        json!({
                            "org_id": repo.org_id.clone(),
                            "project_id": repo.project_id.clone(),
                            "repo_id": repo.id.clone(),
                            "repo_path": repo.local_path.clone(),
                            "timestamp": repo.created_at,
                        })
                        .to_string(),
                    ),
                    SqlParam::I64(repo.created_at),
                    SqlParam::I64(repo.updated_at),
                ],
            )
            .await
    }

    /// Retrieves a repository by ID.
    async fn get_repository(&self, org_id: &str, id: &str) -> Result<Repository> {
        let repo = query_helpers::query_one(
            &self.executor,
            "SELECT * FROM repositories WHERE org_id = ? AND id = ?",
            &[
                SqlParam::String(org_id.to_owned()),
                SqlParam::String(id.to_owned()),
            ],
            Repository::from_row,
        )
        .await?;
        Error::not_found_or(repo, "Repository", id)
    }

    /// Lists repositories in an organization for a project.
    async fn list_repositories(&self, org_id: &str, project_id: &str) -> Result<Vec<Repository>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM repositories WHERE org_id = ? AND project_id = ?",
            &[
                SqlParam::String(org_id.to_owned()),
                SqlParam::String(project_id.to_owned()),
            ],
            Repository::from_row,
            "vcs entity",
        )
        .await
    }

    /// Updates an existing repository.
    async fn update_repository(&self, repo: &Repository) -> Result<()> {
        self.executor
            .execute(
                "UPDATE repositories SET name = ?, url = ?, local_path = ?, vcs_type = ?, origin_context = ?, updated_at = ? WHERE org_id = ? AND id = ?",
                &[
                    SqlParam::String(repo.name.clone()),
                    SqlParam::String(repo.url.clone()),
                    SqlParam::String(repo.local_path.clone()),
                    SqlParam::String(repo.vcs_type.to_string()),
                    SqlParam::String(
                        json!({
                            "org_id": repo.org_id.clone(),
                            "project_id": repo.project_id.clone(),
                            "repo_id": repo.id.clone(),
                            "repo_path": repo.local_path.clone(),
                            "timestamp": repo.updated_at,
                        })
                        .to_string(),
                    ),
                    SqlParam::I64(repo.updated_at),
                    SqlParam::String(repo.org_id.clone()),
                    SqlParam::String(repo.id.clone()),
                ],
            )
            .await
    }

    /// Deletes a repository.
    async fn delete_repository(&self, org_id: &str, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM repositories WHERE org_id = ? AND id = ?",
                &[
                    SqlParam::String(org_id.to_owned()),
                    SqlParam::String(id.to_owned()),
                ],
            )
            .await
    }
    /// Creates a new branch.
    async fn create_branch(&self, branch: &Branch) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO branches (id, org_id, repository_id, name, is_default, head_commit, upstream, origin_context, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(branch.id.clone()),
                    SqlParam::String(branch.org_id.clone()),
                    SqlParam::String(branch.repository_id.clone()),
                    SqlParam::String(branch.name.clone()),
                    SqlParam::I64(if branch.is_default { 1 } else { 0 }),
                    SqlParam::String(branch.head_commit.clone()),
                    match &branch.upstream {
                        Some(u) => SqlParam::String(u.clone()),
                        None => SqlParam::Null,
                    },
                    SqlParam::String(
                        json!({
                            "repository_id": branch.repository_id.clone(),
                            "branch": branch.name.clone(),
                            "commit": branch.head_commit.clone(),
                            "timestamp": branch.created_at,
                        })
                        .to_string(),
                    ),
                    SqlParam::I64(branch.created_at),
                ],
            )
            .await
    }

    /// Retrieves a branch by ID.
    async fn get_branch(&self, id: &str) -> Result<Branch> {
        let branch = query_helpers::query_one(
            &self.executor,
            "SELECT * FROM branches WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            Branch::from_row,
        )
        .await?;
        Error::not_found_or(branch, "Branch", id)
    }

    /// Lists branches in a repository.
    async fn list_branches(&self, repository_id: &str) -> Result<Vec<Branch>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM branches WHERE repository_id = ?",
            &[SqlParam::String(repository_id.to_owned())],
            Branch::from_row,
            "vcs entity",
        )
        .await
    }

    /// Updates an existing branch.
    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        self.executor
            .execute(
                "UPDATE branches SET name = ?, is_default = ?, head_commit = ?, upstream = ?, origin_context = ? WHERE id = ?",
                &[
                    SqlParam::String(branch.name.clone()),
                    SqlParam::I64(if branch.is_default { 1 } else { 0 }),
                    SqlParam::String(branch.head_commit.clone()),
                    match &branch.upstream {
                        Some(u) => SqlParam::String(u.clone()),
                        None => SqlParam::Null,
                    },
                    SqlParam::String(
                        json!({
                            "repository_id": branch.repository_id.clone(),
                            "branch": branch.name.clone(),
                            "commit": branch.head_commit.clone(),
                            "timestamp": branch.created_at,
                        })
                        .to_string(),
                    ),
                    SqlParam::String(branch.id.clone()),
                ],
            )
            .await
    }

    /// Deletes a branch.
    async fn delete_branch(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM branches WHERE id = ?",
                &[SqlParam::String(id.to_owned())],
            )
            .await
    }
    /// Creates a new worktree.
    async fn create_worktree(&self, wt: &Worktree) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO worktrees (id, repository_id, branch_id, path, status, assigned_agent_id, origin_context, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(wt.id.clone()),
                    SqlParam::String(wt.repository_id.clone()),
                    SqlParam::String(wt.branch_id.clone()),
                    SqlParam::String(wt.path.clone()),
                    SqlParam::String(wt.status.to_string()),
                    match &wt.assigned_agent_id {
                        Some(a) => SqlParam::String(a.clone()),
                        None => SqlParam::Null,
                    },
                    SqlParam::String(
                        json!({
                            "repository_id": wt.repository_id.clone(),
                            "worktree_id": wt.id.clone(),
                            "file_path": wt.path.clone(),
                            "timestamp": wt.created_at,
                        })
                        .to_string(),
                    ),
                    SqlParam::I64(wt.created_at),
                    SqlParam::I64(wt.updated_at),
                ],
            )
            .await
    }

    /// Retrieves a worktree by ID.
    async fn get_worktree(&self, id: &str) -> Result<Worktree> {
        let wt = query_helpers::query_one(
            &self.executor,
            "SELECT * FROM worktrees WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            Worktree::from_row,
        )
        .await?;
        Error::not_found_or(wt, "Worktree", id)
    }

    /// Lists worktrees in a repository.
    async fn list_worktrees(&self, repository_id: &str) -> Result<Vec<Worktree>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM worktrees WHERE repository_id = ?",
            &[SqlParam::String(repository_id.to_owned())],
            Worktree::from_row,
            "vcs entity",
        )
        .await
    }

    /// Updates an existing worktree.
    async fn update_worktree(&self, wt: &Worktree) -> Result<()> {
        self.executor
            .execute(
                "UPDATE worktrees SET branch_id = ?, path = ?, status = ?, assigned_agent_id = ?, origin_context = ?, updated_at = ? WHERE id = ?",
                &[
                    SqlParam::String(wt.branch_id.clone()),
                    SqlParam::String(wt.path.clone()),
                    SqlParam::String(wt.status.to_string()),
                    match &wt.assigned_agent_id {
                        Some(a) => SqlParam::String(a.clone()),
                        None => SqlParam::Null,
                    },
                    SqlParam::String(
                        json!({
                            "repository_id": wt.repository_id.clone(),
                            "worktree_id": wt.id.clone(),
                            "file_path": wt.path.clone(),
                            "timestamp": wt.updated_at,
                        })
                        .to_string(),
                    ),
                    SqlParam::I64(wt.updated_at),
                    SqlParam::String(wt.id.clone()),
                ],
            )
            .await
    }

    /// Deletes a worktree.
    async fn delete_worktree(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM worktrees WHERE id = ?",
                &[SqlParam::String(id.to_owned())],
            )
            .await
    }
    /// Creates a new assignment.
    async fn create_assignment(&self, asgn: &AgentWorktreeAssignment) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO agent_worktree_assignments (id, agent_session_id, worktree_id, assigned_at, released_at, origin_context) VALUES (?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(asgn.id.clone()),
                    SqlParam::String(asgn.agent_session_id.clone()),
                    SqlParam::String(asgn.worktree_id.clone()),
                    SqlParam::I64(asgn.assigned_at),
                    match asgn.released_at {
                        Some(r) => SqlParam::I64(r),
                        None => SqlParam::Null,
                    },
                    SqlParam::String(
                        json!({
                            "session_id": asgn.agent_session_id.clone(),
                            "worktree_id": asgn.worktree_id.clone(),
                            "timestamp": asgn.assigned_at,
                        })
                        .to_string(),
                    ),
                ],
            )
            .await
    }

    /// Retrieves an assignment by ID.
    async fn get_assignment(&self, id: &str) -> Result<AgentWorktreeAssignment> {
        let asgn = query_helpers::query_one(
            &self.executor,
            "SELECT * FROM agent_worktree_assignments WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            AgentWorktreeAssignment::from_row,
        )
        .await?;
        Error::not_found_or(asgn, "Assignment", id)
    }

    /// Lists assignments for a worktree.
    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>> {
        query_helpers::query_all(
            &self.executor,
            "SELECT * FROM agent_worktree_assignments WHERE worktree_id = ?",
            &[SqlParam::String(worktree_id.to_owned())],
            AgentWorktreeAssignment::from_row,
            "vcs entity",
        )
        .await
    }

    /// Releases an assignment (sets `released_at`).
    async fn release_assignment(&self, id: &str, released_at: i64) -> Result<()> {
        self.executor
            .execute(
                "UPDATE agent_worktree_assignments SET released_at = ?, origin_context = ? WHERE id = ?",
                &[
                    SqlParam::I64(released_at),
                    SqlParam::String(
                        json!({
                            "assignment_id": id,
                            "timestamp": released_at,
                        })
                        .to_string(),
                    ),
                    SqlParam::String(id.to_owned()),
                ],
            )
            .await
    }
}
