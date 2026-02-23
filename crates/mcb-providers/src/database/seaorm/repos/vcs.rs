#![allow(clippy::missing_errors_doc)]

use std::collections::HashMap;

use async_trait::async_trait;
use mcb_domain::entities::repository::{Branch, Repository};
use mcb_domain::entities::worktree::{AgentWorktreeAssignment, Worktree};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::VcsEntityRepository;
use sea_orm::ActiveValue::Set;
use sea_orm::sea_query::{Alias, Expr, ExprTrait, JoinType, Order, Query};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::database::seaorm::entities::{agent_worktree_assignment, branch, repository, worktree};

pub struct SeaOrmVcsEntityRepository {
    db: DatabaseConnection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchComparison {
    pub repository_id: String,
    pub base_branch: String,
    pub target_branch: String,
    pub base_head_commit: Option<String>,
    pub target_head_commit: Option<String>,
    pub heads_equal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub repository_id: String,
    pub branch_worktree_counts: HashMap<String, i64>,
    pub total_worktrees: i64,
}

impl SeaOrmVcsEntityRepository {
    #[must_use]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn db_error(context: &str, error: impl std::fmt::Display) -> Error {
        Error::database(format!("{context}: {error}"))
    }

    pub async fn index_repository(&self, repo: &Repository) -> Result<()> {
        let existing = repository::Entity::find_by_id(repo.id.clone())
            .one(&self.db)
            .await
            .map_err(|e| Self::db_error("check repository existence", e))?;

        if existing.is_some() {
            return self.update_repository(repo).await;
        }

        self.create_repository(repo).await
    }

    pub async fn search_branch(
        &self,
        repository_id: &str,
        query: &str,
        limit: u64,
    ) -> Result<Vec<Branch>> {
        let like_pattern = format!("%{query}%");
        let branches = Alias::new("branches");
        let select = Query::select()
            .columns([
                Alias::new("id"),
                Alias::new("org_id"),
                Alias::new("repository_id"),
                Alias::new("name"),
                Alias::new("is_default"),
                Alias::new("head_commit"),
                Alias::new("upstream"),
                Alias::new("created_at"),
            ])
            .from(branches.clone())
            .and_where(Expr::col((branches.clone(), Alias::new("repository_id"))).eq(repository_id))
            .and_where(Expr::col((branches.clone(), Alias::new("name"))).like(like_pattern))
            .order_by((branches, Alias::new("name")), Order::Asc)
            .limit(limit)
            .to_owned();

        let stmt = self.db.get_database_backend().build(&select);
        let rows = self
            .db
            .query_all_raw(stmt)
            .await
            .map_err(|e| Self::db_error("search branches", e))?;

        rows.into_iter()
            .map(|row| {
                let is_default = row
                    .try_get::<i64>("", "is_default")
                    .map_err(|e| Self::db_error("read branch is_default", e))?;

                Ok(Branch {
                    id: row
                        .try_get("", "id")
                        .map_err(|e| Self::db_error("read branch id", e))?,
                    org_id: row
                        .try_get("", "org_id")
                        .map_err(|e| Self::db_error("read branch org_id", e))?,
                    repository_id: row
                        .try_get("", "repository_id")
                        .map_err(|e| Self::db_error("read branch repository_id", e))?,
                    name: row
                        .try_get("", "name")
                        .map_err(|e| Self::db_error("read branch name", e))?,
                    is_default: is_default != 0,
                    head_commit: row
                        .try_get("", "head_commit")
                        .map_err(|e| Self::db_error("read branch head_commit", e))?,
                    upstream: row
                        .try_get("", "upstream")
                        .map_err(|e| Self::db_error("read branch upstream", e))?,
                    created_at: row
                        .try_get("", "created_at")
                        .map_err(|e| Self::db_error("read branch created_at", e))?,
                })
            })
            .collect()
    }

    pub async fn compare_branches(
        &self,
        repository_id: &str,
        base_branch: &str,
        target_branch: &str,
    ) -> Result<BranchComparison> {
        let branches = Alias::new("branches");
        let select = Query::select()
            .columns([Alias::new("name"), Alias::new("head_commit")])
            .from(branches.clone())
            .and_where(Expr::col((branches.clone(), Alias::new("repository_id"))).eq(repository_id))
            .and_where(
                Expr::col((branches.clone(), Alias::new("name")))
                    .eq(base_branch)
                    .or(Expr::col((branches, Alias::new("name"))).eq(target_branch)),
            )
            .to_owned();

        let stmt = self.db.get_database_backend().build(&select);
        let rows = self
            .db
            .query_all_raw(stmt)
            .await
            .map_err(|e| Self::db_error("compare branches", e))?;

        let mut base_head = None;
        let mut target_head = None;

        for row in rows {
            let branch_name: String = row
                .try_get("", "name")
                .map_err(|e| Self::db_error("read compared branch name", e))?;
            let head_commit: String = row
                .try_get("", "head_commit")
                .map_err(|e| Self::db_error("read compared branch head_commit", e))?;

            if branch_name == base_branch {
                base_head = Some(head_commit);
            } else if branch_name == target_branch {
                target_head = Some(head_commit);
            }
        }

        let heads_equal = base_head.is_some() && base_head == target_head;

        Ok(BranchComparison {
            repository_id: repository_id.to_owned(),
            base_branch: base_branch.to_owned(),
            target_branch: target_branch.to_owned(),
            base_head_commit: base_head,
            target_head_commit: target_head,
            heads_equal,
        })
    }

    pub async fn analyze_impact(
        &self,
        repository_id: &str,
        base_branch: &str,
        target_branch: &str,
    ) -> Result<ImpactAnalysis> {
        let branches = Alias::new("branches");
        let worktrees = Alias::new("worktrees");
        let select = Query::select()
            .expr_as(
                Expr::col((branches.clone(), Alias::new("name"))),
                Alias::new("branch_name"),
            )
            .expr_as(
                Expr::col((worktrees.clone(), Alias::new("id"))).count(),
                Alias::new("worktree_count"),
            )
            .from(branches.clone())
            .join(
                JoinType::LeftJoin,
                worktrees.clone(),
                Expr::col((branches.clone(), Alias::new("id")))
                    .equals((worktrees, Alias::new("branch_id"))),
            )
            .and_where(Expr::col((branches.clone(), Alias::new("repository_id"))).eq(repository_id))
            .and_where(
                Expr::col((branches.clone(), Alias::new("name")))
                    .eq(base_branch)
                    .or(Expr::col((branches, Alias::new("name"))).eq(target_branch)),
            )
            .group_by_col((Alias::new("branches"), Alias::new("name")))
            .to_owned();

        let stmt = self.db.get_database_backend().build(&select);
        let rows = self
            .db
            .query_all_raw(stmt)
            .await
            .map_err(|e| Self::db_error("analyze branch impact", e))?;

        let mut branch_worktree_counts = HashMap::new();
        let mut total_worktrees = 0_i64;

        for row in rows {
            let branch_name: String = row
                .try_get("", "branch_name")
                .map_err(|e| Self::db_error("read branch_name", e))?;
            let worktree_count: i64 = row
                .try_get("", "worktree_count")
                .map_err(|e| Self::db_error("read worktree_count", e))?;
            total_worktrees += worktree_count;
            branch_worktree_counts.insert(branch_name, worktree_count);
        }

        Ok(ImpactAnalysis {
            repository_id: repository_id.to_owned(),
            branch_worktree_counts,
            total_worktrees,
        })
    }
}

#[async_trait]
impl VcsEntityRepository for SeaOrmVcsEntityRepository {
    async fn create_repository(&self, repo: &Repository) -> Result<()> {
        let mut active: repository::ActiveModel = repo.clone().into();
        active.origin_context = Set(Some(
            json!({
                "org_id": repo.org_id,
                "project_id": repo.project_id,
                "repo_id": repo.id,
                "repo_path": repo.local_path,
                "timestamp": repo.created_at,
            })
            .to_string(),
        ));
        active
            .insert(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Self::db_error("create repository", e))
    }

    async fn get_repository(&self, org_id: &str, id: &str) -> Result<Repository> {
        let repository = repository::Entity::find()
            .filter(repository::Column::OrgId.eq(org_id.to_owned()))
            .filter(repository::Column::Id.eq(id.to_owned()))
            .one(&self.db)
            .await
            .map_err(|e| Self::db_error("get repository", e))?
            .map(Repository::from);
        Error::not_found_or(repository, "Repository", id)
    }

    async fn list_repositories(&self, org_id: &str, project_id: &str) -> Result<Vec<Repository>> {
        let mut query = repository::Entity::find().filter(repository::Column::OrgId.eq(org_id));
        if !project_id.trim().is_empty() {
            query = query.filter(repository::Column::ProjectId.eq(project_id));
        }

        query
            .all(&self.db)
            .await
            .map_err(|e| Self::db_error("list repositories", e))
            .map(|rows| rows.into_iter().map(Repository::from).collect())
    }

    async fn update_repository(&self, repo: &Repository) -> Result<()> {
        let mut active: repository::ActiveModel = repo.clone().into();
        active.origin_context = Set(Some(
            json!({
                "org_id": repo.org_id,
                "project_id": repo.project_id,
                "repo_id": repo.id,
                "repo_path": repo.local_path,
                "timestamp": repo.updated_at,
            })
            .to_string(),
        ));
        active
            .update(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Self::db_error("update repository", e))
    }

    async fn delete_repository(&self, org_id: &str, id: &str) -> Result<()> {
        repository::Entity::delete_many()
            .filter(repository::Column::OrgId.eq(org_id.to_owned()))
            .filter(repository::Column::Id.eq(id.to_owned()))
            .exec(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Self::db_error("delete repository", e))
    }

    async fn create_branch(&self, branch_entity: &Branch) -> Result<()> {
        let mut active: branch::ActiveModel = branch_entity.clone().into();
        active.origin_context = Set(Some(
            json!({
                "repository_id": branch_entity.repository_id,
                "branch": branch_entity.name,
                "commit": branch_entity.head_commit,
                "timestamp": branch_entity.created_at,
            })
            .to_string(),
        ));
        active
            .insert(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Self::db_error("create branch", e))
    }

    async fn get_branch(&self, id: &str) -> Result<Branch> {
        let branch_entity = branch::Entity::find_by_id(id.to_owned())
            .one(&self.db)
            .await
            .map_err(|e| Self::db_error("get branch", e))?
            .map(Branch::from);
        Error::not_found_or(branch_entity, "Branch", id)
    }

    async fn list_branches(&self, repository_id: &str) -> Result<Vec<Branch>> {
        branch::Entity::find()
            .filter(branch::Column::RepositoryId.eq(repository_id.to_owned()))
            .all(&self.db)
            .await
            .map_err(|e| Self::db_error("list branches", e))
            .map(|rows| rows.into_iter().map(Branch::from).collect())
    }

    async fn update_branch(&self, branch_entity: &Branch) -> Result<()> {
        let mut active: branch::ActiveModel = branch_entity.clone().into();
        active.origin_context = Set(Some(
            json!({
                "repository_id": branch_entity.repository_id,
                "branch": branch_entity.name,
                "commit": branch_entity.head_commit,
                "timestamp": branch_entity.created_at,
            })
            .to_string(),
        ));
        active
            .update(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Self::db_error("update branch", e))
    }

    async fn delete_branch(&self, id: &str) -> Result<()> {
        branch::Entity::delete_many()
            .filter(branch::Column::Id.eq(id.to_owned()))
            .exec(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Self::db_error("delete branch", e))
    }

    async fn create_worktree(&self, worktree_entity: &Worktree) -> Result<()> {
        let mut active: worktree::ActiveModel = worktree_entity.clone().into();
        active.origin_context = Set(Some(
            json!({
                "repository_id": worktree_entity.repository_id,
                "worktree_id": worktree_entity.id,
                "file_path": worktree_entity.path,
                "timestamp": worktree_entity.created_at,
            })
            .to_string(),
        ));
        active
            .insert(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Self::db_error("create worktree", e))
    }

    async fn get_worktree(&self, id: &str) -> Result<Worktree> {
        let worktree_entity = worktree::Entity::find_by_id(id.to_owned())
            .one(&self.db)
            .await
            .map_err(|e| Self::db_error("get worktree", e))?
            .map(Worktree::from);
        Error::not_found_or(worktree_entity, "Worktree", id)
    }

    async fn list_worktrees(&self, repository_id: &str) -> Result<Vec<Worktree>> {
        worktree::Entity::find()
            .filter(worktree::Column::RepositoryId.eq(repository_id.to_owned()))
            .all(&self.db)
            .await
            .map_err(|e| Self::db_error("list worktrees", e))
            .map(|rows| rows.into_iter().map(Worktree::from).collect())
    }

    async fn update_worktree(&self, worktree_entity: &Worktree) -> Result<()> {
        let mut active: worktree::ActiveModel = worktree_entity.clone().into();
        active.origin_context = Set(Some(
            json!({
                "repository_id": worktree_entity.repository_id,
                "worktree_id": worktree_entity.id,
                "file_path": worktree_entity.path,
                "timestamp": worktree_entity.updated_at,
            })
            .to_string(),
        ));
        active
            .update(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Self::db_error("update worktree", e))
    }

    async fn delete_worktree(&self, id: &str) -> Result<()> {
        worktree::Entity::delete_many()
            .filter(worktree::Column::Id.eq(id.to_owned()))
            .exec(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Self::db_error("delete worktree", e))
    }

    async fn create_assignment(&self, assignment: &AgentWorktreeAssignment) -> Result<()> {
        let mut active: agent_worktree_assignment::ActiveModel = assignment.clone().into();
        active.origin_context = Set(Some(
            json!({
                "session_id": assignment.agent_session_id,
                "worktree_id": assignment.worktree_id,
                "timestamp": assignment.assigned_at,
            })
            .to_string(),
        ));
        active
            .insert(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Self::db_error("create assignment", e))
    }

    async fn get_assignment(&self, id: &str) -> Result<AgentWorktreeAssignment> {
        let assignment = agent_worktree_assignment::Entity::find_by_id(id.to_owned())
            .one(&self.db)
            .await
            .map_err(|e| Self::db_error("get assignment", e))?
            .map(AgentWorktreeAssignment::from);
        Error::not_found_or(assignment, "Assignment", id)
    }

    async fn list_assignments_by_worktree(
        &self,
        worktree_id: &str,
    ) -> Result<Vec<AgentWorktreeAssignment>> {
        agent_worktree_assignment::Entity::find()
            .filter(agent_worktree_assignment::Column::WorktreeId.eq(worktree_id.to_owned()))
            .all(&self.db)
            .await
            .map_err(|e| Self::db_error("list assignments by worktree", e))
            .map(|rows| {
                rows.into_iter()
                    .map(AgentWorktreeAssignment::from)
                    .collect()
            })
    }

    async fn release_assignment(&self, id: &str, released_at: i64) -> Result<()> {
        let mut assignment = agent_worktree_assignment::Entity::find_by_id(id.to_owned())
            .one(&self.db)
            .await
            .map_err(|e| Self::db_error("load assignment to release", e))
            .and_then(|opt| Error::not_found_or(opt, "Assignment", id))?
            .into_active_model();

        assignment.released_at = Set(Some(released_at));
        assignment.origin_context = Set(Some(
            json!({
                "assignment_id": id,
                "timestamp": released_at,
            })
            .to_string(),
        ));

        assignment
            .update(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| Self::db_error("release assignment", e))
    }
}
