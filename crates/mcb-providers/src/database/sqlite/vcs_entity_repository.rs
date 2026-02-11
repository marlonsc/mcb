use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::repository::{Branch, Repository, VcsType};
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree, WorktreeStatus};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam, SqlRow};
use mcb_domain::ports::repositories::VcsEntityRepository;

pub struct SqliteVcsEntityRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

impl SqliteVcsEntityRepository {
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }

    async fn query_one<T, F>(&self, sql: &str, params: &[SqlParam], convert: F) -> Result<Option<T>>
    where
        F: FnOnce(&dyn SqlRow) -> Result<T>,
    {
        match self.executor.query_one(sql, params).await? {
            Some(r) => Ok(Some(convert(r.as_ref())?)),
            None => Ok(None),
        }
    }

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

fn row_to_repository(row: &dyn SqlRow) -> Result<Repository> {
    let vcs_type = req_str(row, "vcs_type")?
        .parse::<VcsType>()
        .map_err(|e| Error::memory(format!("Invalid vcs_type: {e}")))?;

    Ok(Repository {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        project_id: req_str(row, "project_id")?,
        name: req_str(row, "name")?,
        url: req_str(row, "url")?,
        local_path: req_str(row, "local_path")?,
        vcs_type,
        created_at: req_i64(row, "created_at")?,
        updated_at: req_i64(row, "updated_at")?,
    })
}

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

fn row_to_worktree(row: &dyn SqlRow) -> Result<Worktree> {
    let status = req_str(row, "status")?
        .parse::<WorktreeStatus>()
        .map_err(|e| Error::memory(format!("Invalid worktree status: {e}")))?;

    Ok(Worktree {
        id: req_str(row, "id")?,
        repository_id: req_str(row, "repository_id")?,
        branch_id: req_str(row, "branch_id")?,
        path: req_str(row, "path")?,
        status,
        assigned_agent_id: row.try_get_string("assigned_agent_id")?,
        created_at: req_i64(row, "created_at")?,
        updated_at: req_i64(row, "updated_at")?,
    })
}

fn row_to_assignment(row: &dyn SqlRow) -> Result<AgentWorktreeAssignment> {
    Ok(AgentWorktreeAssignment {
        id: req_str(row, "id")?,
        agent_session_id: req_str(row, "agent_session_id")?,
        worktree_id: req_str(row, "worktree_id")?,
        assigned_at: req_i64(row, "assigned_at")?,
        released_at: row.try_get_i64("released_at")?,
    })
}

fn req_str(row: &dyn SqlRow, col: &str) -> Result<String> {
    row.try_get_string(col)?
        .ok_or_else(|| Error::memory(format!("Missing {col}")))
}

fn req_i64(row: &dyn SqlRow, col: &str) -> Result<i64> {
    row.try_get_i64(col)?
        .ok_or_else(|| Error::memory(format!("Missing {col}")))
}

#[async_trait]
impl VcsEntityRepository for SqliteVcsEntityRepository {
    // -- Repository --

    async fn create_repository(&self, repo: &Repository) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO repositories (id, org_id, project_id, name, url, local_path, vcs_type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(repo.id.clone()),
                    SqlParam::String(repo.org_id.clone()),
                    SqlParam::String(repo.project_id.clone()),
                    SqlParam::String(repo.name.clone()),
                    SqlParam::String(repo.url.clone()),
                    SqlParam::String(repo.local_path.clone()),
                    SqlParam::String(repo.vcs_type.to_string()),
                    SqlParam::I64(repo.created_at),
                    SqlParam::I64(repo.updated_at),
                ],
            )
            .await
    }

    async fn get_repository(&self, org_id: &str, id: &str) -> Result<Option<Repository>> {
        self.query_one(
            "SELECT * FROM repositories WHERE org_id = ? AND id = ?",
            &[
                SqlParam::String(org_id.to_string()),
                SqlParam::String(id.to_string()),
            ],
            row_to_repository,
        )
        .await
    }

    async fn list_repositories(&self, org_id: &str, project_id: &str) -> Result<Vec<Repository>> {
        self.query_all(
            "SELECT * FROM repositories WHERE org_id = ? AND project_id = ?",
            &[
                SqlParam::String(org_id.to_string()),
                SqlParam::String(project_id.to_string()),
            ],
            row_to_repository,
        )
        .await
    }

    async fn update_repository(&self, repo: &Repository) -> Result<()> {
        self.executor
            .execute(
                "UPDATE repositories SET name = ?, url = ?, local_path = ?, vcs_type = ?, updated_at = ? WHERE org_id = ? AND id = ?",
                &[
                    SqlParam::String(repo.name.clone()),
                    SqlParam::String(repo.url.clone()),
                    SqlParam::String(repo.local_path.clone()),
                    SqlParam::String(repo.vcs_type.to_string()),
                    SqlParam::I64(repo.updated_at),
                    SqlParam::String(repo.org_id.clone()),
                    SqlParam::String(repo.id.clone()),
                ],
            )
            .await
    }

    async fn delete_repository(&self, org_id: &str, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM repositories WHERE org_id = ? AND id = ?",
                &[
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String(id.to_string()),
                ],
            )
            .await
    }

    // -- Branch --

    async fn create_branch(&self, branch: &Branch) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO branches (id, repository_id, name, is_default, head_commit, upstream, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
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
                    SqlParam::I64(branch.created_at),
                ],
            )
            .await
    }

    async fn get_branch(&self, id: &str) -> Result<Option<Branch>> {
        self.query_one(
            "SELECT * FROM branches WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_branch,
        )
        .await
    }

    async fn list_branches(&self, repository_id: &str) -> Result<Vec<Branch>> {
        self.query_all(
            "SELECT * FROM branches WHERE repository_id = ?",
            &[SqlParam::String(repository_id.to_string())],
            row_to_branch,
        )
        .await
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        self.executor
            .execute(
                "UPDATE branches SET name = ?, is_default = ?, head_commit = ?, upstream = ? WHERE id = ?",
                &[
                    SqlParam::String(branch.name.clone()),
                    SqlParam::I64(if branch.is_default { 1 } else { 0 }),
                    SqlParam::String(branch.head_commit.clone()),
                    match &branch.upstream {
                        Some(u) => SqlParam::String(u.clone()),
                        None => SqlParam::Null,
                    },
                    SqlParam::String(branch.id.clone()),
                ],
            )
            .await
    }

    async fn delete_branch(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM branches WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await
    }

    // -- Worktree --

    async fn create_worktree(&self, wt: &Worktree) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO worktrees (id, repository_id, branch_id, path, status, assigned_agent_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
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
                    SqlParam::I64(wt.created_at),
                    SqlParam::I64(wt.updated_at),
                ],
            )
            .await
    }

    async fn get_worktree(&self, id: &str) -> Result<Option<Worktree>> {
        self.query_one(
            "SELECT * FROM worktrees WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_worktree,
        )
        .await
    }

    async fn list_worktrees(&self, repository_id: &str) -> Result<Vec<Worktree>> {
        self.query_all(
            "SELECT * FROM worktrees WHERE repository_id = ?",
            &[SqlParam::String(repository_id.to_string())],
            row_to_worktree,
        )
        .await
    }

    async fn update_worktree(&self, wt: &Worktree) -> Result<()> {
        self.executor
            .execute(
                "UPDATE worktrees SET branch_id = ?, path = ?, status = ?, assigned_agent_id = ?, updated_at = ? WHERE id = ?",
                &[
                    SqlParam::String(wt.branch_id.clone()),
                    SqlParam::String(wt.path.clone()),
                    SqlParam::String(wt.status.to_string()),
                    match &wt.assigned_agent_id {
                        Some(a) => SqlParam::String(a.clone()),
                        None => SqlParam::Null,
                    },
                    SqlParam::I64(wt.updated_at),
                    SqlParam::String(wt.id.clone()),
                ],
            )
            .await
    }

    async fn delete_worktree(&self, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM worktrees WHERE id = ?",
                &[SqlParam::String(id.to_string())],
            )
            .await
    }

    // -- Assignment --

    async fn create_assignment(&self, asgn: &AgentWorktreeAssignment) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO agent_worktree_assignments (id, agent_session_id, worktree_id, assigned_at, released_at) VALUES (?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(asgn.id.clone()),
                    SqlParam::String(asgn.agent_session_id.clone()),
                    SqlParam::String(asgn.worktree_id.clone()),
                    SqlParam::I64(asgn.assigned_at),
                    match asgn.released_at {
                        Some(r) => SqlParam::I64(r),
                        None => SqlParam::Null,
                    },
                ],
            )
            .await
    }

    async fn get_assignment(&self, id: &str) -> Result<Option<AgentWorktreeAssignment>> {
        self.query_one(
            "SELECT * FROM agent_worktree_assignments WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_assignment,
        )
        .await
    }

    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>> {
        self.query_all(
            "SELECT * FROM agent_worktree_assignments WHERE worktree_id = ?",
            &[SqlParam::String(worktree_id.to_string())],
            row_to_assignment,
        )
        .await
    }

    async fn release_assignment(&self, id: &str, released_at: i64) -> Result<()> {
        self.executor
            .execute(
                "UPDATE agent_worktree_assignments SET released_at = ? WHERE id = ?",
                &[SqlParam::I64(released_at), SqlParam::String(id.to_string())],
            )
            .await
    }
}
