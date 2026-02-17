//! `SQLite` VCS entity repository.

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::repository::{Branch, Repository, VcsType};
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree, WorktreeStatus};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{AssignmentManager, BranchRegistry, RepositoryRegistry, WorktreeManager};
use mcb_domain::ports::{DatabaseExecutor, SqlParam, SqlRow};
use serde_json::json;

use crate::utils::sqlite::row::{req_i64, req_str};

/// SQLite-backed repository for VCS repositories, branches, worktrees, and assignments.
pub struct SqliteVcsEntityRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

impl SqliteVcsEntityRepository {
    /// Creates a new repository using the provided database executor.
    // TODO(qlty): Found 31 lines of similar code in 3 locations (mass = 216)
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }

    /// Helper to query a single row and convert it.
    async fn query_one<T, F>(&self, sql: &str, params: &[SqlParam], convert: F) -> Result<Option<T>>
    where
        F: FnOnce(&dyn SqlRow) -> Result<T>,
    {
        match self.executor.query_one(sql, params).await? {
            Some(r) => Ok(Some(convert(r.as_ref())?)),
            None => Ok(None),
        }
    }

    /// Helper to query multiple rows and convert them.
    async fn query_all<T, F>(&self, sql: &str, params: &[SqlParam], convert: F) -> Result<Vec<T>>
    where
        F: Fn(&dyn SqlRow) -> Result<T>,
    {
        let rows = self.executor.query_all(sql, params).await?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(
                convert(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode vcs entity", e))?,
            );
        }
        Ok(result)
    }
}

/// Converts a SQL row to a Repository.
fn row_to_repository(row: &dyn SqlRow) -> Result<Repository> {
    let vcs_type = req_str(row, "vcs_type")?
        .parse::<VcsType>()
        .map_err(|e| Error::memory(format!("Invalid vcs_type: {e}")))?;

    Ok(Repository {
        metadata: mcb_domain::entities::EntityMetadata {
            id: req_str(row, "id")?,
            created_at: req_i64(row, "created_at")?,
            updated_at: req_i64(row, "updated_at")?,
        },
        org_id: req_str(row, "org_id")?,
        project_id: req_str(row, "project_id")?,
        name: req_str(row, "name")?,
        url: req_str(row, "url")?,
        local_path: req_str(row, "local_path")?,
        vcs_type,
    })
}

/// Converts a SQL row to a Branch.
fn row_to_branch(row: &dyn SqlRow) -> Result<Branch> {
    let is_default_i = req_i64(row, "is_default")?;
    Ok(Branch {
        id: req_str(row, "id")?,
        repository_id: req_str(row, "repository_id")?,
        name: req_str(row, "name")?,
        is_default: is_default_i != 0,
        head_commit: req_str(row, "head_commit")?,
        upstream: row.try_get_string("upstream")?,
        created_at: req_i64(row, "created_at")?,
    })
}

/// Converts a SQL row to a Worktree.
fn row_to_worktree(row: &dyn SqlRow) -> Result<Worktree> {
    let status = req_str(row, "status")?
        .parse::<WorktreeStatus>()
        .map_err(|e| Error::memory(format!("Invalid worktree status: {e}")))?;

    Ok(Worktree {
        metadata: mcb_domain::entities::EntityMetadata {
            id: req_str(row, "id")?,
            created_at: req_i64(row, "created_at")?,
            updated_at: req_i64(row, "updated_at")?,
        },
        repository_id: req_str(row, "repository_id")?,
        branch_id: req_str(row, "branch_id")?,
        path: req_str(row, "path")?,
        status,
        assigned_agent_id: row.try_get_string("assigned_agent_id")?,
    })
}

/// Converts a SQL row to an `AgentWorktreeAssignment`.
fn row_to_assignment(row: &dyn SqlRow) -> Result<AgentWorktreeAssignment> {
    Ok(AgentWorktreeAssignment {
        id: req_str(row, "id")?,
        agent_session_id: req_str(row, "agent_session_id")?,
        worktree_id: req_str(row, "worktree_id")?,
        assigned_at: req_i64(row, "assigned_at")?,
        released_at: row.try_get_i64("released_at")?,
    })
}

#[async_trait]
/// Registry for VCS repositories.
impl RepositoryRegistry for SqliteVcsEntityRepository {
    // -- Repository --

    /// Creates a new repository.
    async fn create_repository(&self, repo: &Repository) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO repositories (id, org_id, project_id, name, url, local_path, vcs_type, origin_context, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(repo.metadata.id.clone()),
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
                            "repo_id": repo.metadata.id.clone(),
                            "repo_path": repo.local_path.clone(),
                            "timestamp": repo.metadata.created_at,
                        })
                        .to_string(),
                    ),
                    SqlParam::I64(repo.metadata.created_at),
                    SqlParam::I64(repo.metadata.updated_at),
                ],
            )
            .await
    }

    /// Retrieves a repository by ID.
    async fn get_repository(&self, org_id: &str, id: &str) -> Result<Repository> {
        self.query_one(
            "SELECT * FROM repositories WHERE org_id = ? AND id = ?",
            &[
                SqlParam::String(org_id.to_owned()),
                SqlParam::String(id.to_owned()),
            ],
            row_to_repository,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Repository {id}")))
    }

    /// Lists repositories in an organization for a project.
    async fn list_repositories(&self, org_id: &str, project_id: &str) -> Result<Vec<Repository>> {
        self.query_all(
            "SELECT * FROM repositories WHERE org_id = ? AND project_id = ?",
            &[
                SqlParam::String(org_id.to_owned()),
                SqlParam::String(project_id.to_owned()),
            ],
            row_to_repository,
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
                            "repo_id": repo.metadata.id.clone(),
                            "repo_path": repo.local_path.clone(),
                            "timestamp": repo.metadata.updated_at,
                        })
                        .to_string(),
                    ),
                    SqlParam::I64(repo.metadata.updated_at),
                    SqlParam::String(repo.org_id.clone()),
                    SqlParam::String(repo.metadata.id.clone()),
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
}

#[async_trait]
/// Registry for branches.
impl BranchRegistry for SqliteVcsEntityRepository {
    /// Creates a new branch.
    async fn create_branch(&self, branch: &Branch) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO branches (id, repository_id, name, is_default, head_commit, upstream, origin_context, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(branch.id.clone()),
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
        self.query_one(
            "SELECT * FROM branches WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            row_to_branch,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Branch {id}")))
    }

    /// Lists branches in a repository.
    async fn list_branches(&self, repository_id: &str) -> Result<Vec<Branch>> {
        self.query_all(
            "SELECT * FROM branches WHERE repository_id = ?",
            &[SqlParam::String(repository_id.to_owned())],
            row_to_branch,
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
}

#[async_trait]
/// Manager for worktrees.
impl WorktreeManager for SqliteVcsEntityRepository {
    /// Creates a new worktree.
    async fn create_worktree(&self, wt: &Worktree) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO worktrees (id, repository_id, branch_id, path, status, assigned_agent_id, origin_context, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(wt.metadata.id.clone()),
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
                            "worktree_id": wt.metadata.id.clone(),
                            "file_path": wt.path.clone(),
                            "timestamp": wt.metadata.created_at,
                        })
                        .to_string(),
                    ),
                    SqlParam::I64(wt.metadata.created_at),
                    SqlParam::I64(wt.metadata.updated_at),
                ],
            )
            .await
    }

    /// Retrieves a worktree by ID.
    async fn get_worktree(&self, id: &str) -> Result<Worktree> {
        self.query_one(
            "SELECT * FROM worktrees WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            row_to_worktree,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Worktree {id}")))
    }

    /// Lists worktrees in a repository.
    async fn list_worktrees(&self, repository_id: &str) -> Result<Vec<Worktree>> {
        self.query_all(
            "SELECT * FROM worktrees WHERE repository_id = ?",
            &[SqlParam::String(repository_id.to_owned())],
            row_to_worktree,
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
                            "worktree_id": wt.metadata.id.clone(),
                            "file_path": wt.path.clone(),
                            "timestamp": wt.metadata.updated_at,
                        })
                        .to_string(),
                    ),
                    SqlParam::I64(wt.metadata.updated_at),
                    SqlParam::String(wt.metadata.id.clone()),
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
}

#[async_trait]
/// Manager for agent worktree assignments.
impl AssignmentManager for SqliteVcsEntityRepository {
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
        self.query_one(
            "SELECT * FROM agent_worktree_assignments WHERE id = ?",
            &[SqlParam::String(id.to_owned())],
            row_to_assignment,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Assignment {id}")))
    }

    /// Lists assignments for a worktree.
    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>> {
        self.query_all(
            "SELECT * FROM agent_worktree_assignments WHERE worktree_id = ?",
            &[SqlParam::String(worktree_id.to_owned())],
            row_to_assignment,
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
