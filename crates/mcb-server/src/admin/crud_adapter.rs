//! Generic CRUD adapter that bridges entity handlers with domain services.
//!
//! Each entity slug maps to an adapter implementation that knows how to
//! call the correct service methods and serialize results to JSON.

use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::{
    AgentWorktreeAssignment, ApiKey, Branch, IssueComment, IssueLabel, IssueLabelAssignment,
    Organization, Plan, PlanReview, PlanVersion, ProjectIssue, Repository, Team, TeamMember, User,
    Worktree,
};
use mcb_domain::ports::repositories::{
    IssueEntityRepository, OrgEntityRepository, PlanEntityRepository, VcsEntityRepository,
};
use serde_json::Value;

use super::handlers::AdminState;
use super::web::filter::{
    FilterParams, FilteredResult, SortOrder, parse_iso_date_to_epoch, parse_iso_date_to_epoch_end,
};

/// Async CRUD operations that map entity slugs to domain service calls.
#[async_trait]
pub trait EntityCrudAdapter: Send + Sync {
    /// List all records for this entity.
    async fn list_all(&self) -> Result<Vec<Value>, String>;
    /// Get a single record by its primary key.
    async fn get_by_id(&self, id: &str) -> Result<Value, String>;
    /// Create a record from raw JSON form data.
    async fn create_from_json(&self, data: Value) -> Result<Value, String>;
    /// Update a record from raw JSON form data.
    async fn update_from_json(&self, data: Value) -> Result<(), String>;
    /// Delete a record by its primary key.
    async fn delete_by_id(&self, id: &str) -> Result<(), String>;

    /// List records with in-memory filtering, sorting, and pagination.
    ///
    /// The default implementation fetches all records via [`list_all`](Self::list_all),
    /// then applies search, date-range, sort, and pagination in memory.
    /// `valid_sort_fields` restricts which field names are accepted for sorting;
    /// an unrecognised sort field is silently ignored (no sort applied).
    async fn list_filtered(
        &self,
        params: &FilterParams,
        valid_sort_fields: &HashSet<String>,
    ) -> Result<FilteredResult, String> {
        let records = self.list_all().await?;
        Ok(apply_filter_pipeline(records, params, valid_sort_fields))
    }
}

/// Resolves a CRUD adapter for the given entity slug from AdminState.
pub fn resolve_adapter(slug: &str, state: &AdminState) -> Option<Box<dyn EntityCrudAdapter>> {
    match slug {
        "organizations" => state
            .org_entity
            .as_ref()
            .map(|s| Box::new(OrgAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "users" => state
            .org_entity
            .as_ref()
            .map(|s| Box::new(UserAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "teams" => state
            .org_entity
            .as_ref()
            .map(|s| Box::new(TeamAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "team-members" => state
            .org_entity
            .as_ref()
            .map(|s| Box::new(TeamMemberAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "api-keys" => state
            .org_entity
            .as_ref()
            .map(|s| Box::new(ApiKeyAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "project-issues" => state
            .issue_entity
            .as_ref()
            .map(|s| Box::new(ProjectIssueAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "issue-comments" => state
            .issue_entity
            .as_ref()
            .map(|s| Box::new(IssueCommentAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "issue-labels" => state
            .issue_entity
            .as_ref()
            .map(|s| Box::new(IssueLabelAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "issue-label-assignments" => state.issue_entity.as_ref().map(|s| {
            Box::new(IssueLabelAssignmentAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>
        }),
        "plans" => state
            .plan_entity
            .as_ref()
            .map(|s| Box::new(PlanAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "plan-versions" => state
            .plan_entity
            .as_ref()
            .map(|s| Box::new(PlanVersionAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "plan-reviews" => state
            .plan_entity
            .as_ref()
            .map(|s| Box::new(PlanReviewAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "repositories" => state
            .vcs_entity
            .as_ref()
            .map(|s| Box::new(RepositoryAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "branches" => state
            .vcs_entity
            .as_ref()
            .map(|s| Box::new(BranchAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "worktrees" => state
            .vcs_entity
            .as_ref()
            .map(|s| Box::new(WorktreeAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>),
        "agent-worktree-assignments" => state.vcs_entity.as_ref().map(|s| {
            Box::new(AgentWorktreeAssignmentAdapter(Arc::clone(s))) as Box<dyn EntityCrudAdapter>
        }),
        _ => None,
    }
}

fn to_json<T: serde::Serialize>(val: &T) -> Result<Value, String> {
    serde_json::to_value(val).map_err(|e| e.to_string())
}

fn to_json_vec<T: serde::Serialize>(vals: &[T]) -> Result<Vec<Value>, String> {
    vals.iter().map(|v| to_json(v)).collect()
}

fn from_json<T: serde::de::DeserializeOwned>(data: Value) -> Result<T, String> {
    serde_json::from_value(data).map_err(|e| format!("Invalid data: {e}"))
}

fn map_err(e: mcb_domain::error::Error) -> String {
    e.to_string()
}

fn json_sort_key(v: &Value) -> String {
    match v {
        Value::String(s) => s.to_lowercase(),
        Value::Number(n) => format!("{:020}", n.as_f64().unwrap_or(0.0)),
        Value::Bool(b) => format!("{b}"),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Apply in-memory filtering, sorting, and pagination to a pre-fetched record list.
///
/// Shared by the default `list_filtered()` trait method and by dependent-entity
/// adapters that override `list_filtered()` to first fetch scoped records via
/// a parent-context repository method.
fn apply_filter_pipeline(
    mut records: Vec<Value>,
    params: &FilterParams,
    valid_sort_fields: &HashSet<String>,
) -> FilteredResult {
    // Search
    if let Some(ref q) = params.search
        && !q.is_empty()
    {
        let q_lower = q.to_lowercase();
        records.retain(|rec| {
            if let Value::Object(map) = rec {
                map.values().any(|v| match v {
                    Value::String(s) => s.to_lowercase().contains(&q_lower),
                    _ => v.to_string().to_lowercase().contains(&q_lower),
                })
            } else {
                false
            }
        });
    }

    // Date-range filter — parse ISO strings to epoch for comparison
    let epoch_from = params.date_from.as_deref().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            parse_iso_date_to_epoch(s)
        }
    });
    let epoch_to = params.date_to.as_deref().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            parse_iso_date_to_epoch_end(s)
        }
    });

    if epoch_from.is_some() || epoch_to.is_some() {
        records.retain(|rec| {
            if let Value::Object(map) = rec {
                map.iter()
                    .filter(|(k, _)| k.ends_with("_at"))
                    .any(|(_, v)| {
                        let ts = match v {
                            Value::Number(n) => n.as_i64(),
                            _ => None,
                        };
                        if let Some(ts) = ts {
                            let after = epoch_from.is_none_or(|from| ts >= from);
                            let before = epoch_to.is_none_or(|to| ts <= to);
                            after && before
                        } else {
                            true
                        }
                    })
            } else {
                true
            }
        });
    }

    // Sort
    if let Some(ref field) = params.sort_field
        && valid_sort_fields.contains(field.as_str())
    {
        let desc = matches!(params.sort_order, Some(SortOrder::Desc));
        records.sort_by(|a, b| {
            let va = a.get(field).map(json_sort_key);
            let vb = b.get(field).map(json_sort_key);
            let cmp = va.cmp(&vb);
            if desc { cmp.reverse() } else { cmp }
        });
    }

    // Paginate
    let total_count = records.len();
    let per_page = if params.per_page == 0 {
        20
    } else {
        params.per_page
    };
    let total_pages = if total_count == 0 {
        0
    } else {
        total_count.div_ceil(per_page)
    };
    let page = if params.page == 0 { 1 } else { params.page };
    let start = (page - 1) * per_page;
    let page_records = if start >= total_count {
        Vec::new()
    } else {
        let end = (start + per_page).min(total_count);
        records[start..end].to_vec()
    };

    FilteredResult {
        records: page_records,
        total_count,
        page,
        per_page,
        total_pages,
    }
}

// ─── Org group ───────────────────────────────────────────────────────

impl_crud_adapter!(OrgAdapter(dyn OrgEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        let orgs = self.0.list_orgs().await.map_err(map_err)?;
        to_json_vec(&orgs)
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let org = self.0.get_org(id).await.map_err(map_err)?;
        to_json(&org)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let org: Organization = from_json(data)?;
        self.0.create_org(&org).await.map_err(map_err)?;
        to_json(&org)
    }
    async fn update_from_json(&self, data: Value) -> Result<(), String> {
        let org: Organization = from_json(data)?;
        self.0.update_org(&org).await.map_err(map_err)
    }
    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        self.0.delete_org(id).await.map_err(map_err)
    }
});

impl_crud_adapter!(UserAdapter(dyn OrgEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        let users = self.0.list_users(DEFAULT_ORG_ID).await.map_err(map_err)?;
        to_json_vec(&users)
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let user = self.0.get_user(id).await.map_err(map_err)?;
        to_json(&user)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let user: User = from_json(data)?;
        self.0.create_user(&user).await.map_err(map_err)?;
        to_json(&user)
    }
    async fn update_from_json(&self, data: Value) -> Result<(), String> {
        let user: User = from_json(data)?;
        self.0.update_user(&user).await.map_err(map_err)
    }
    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        self.0.delete_user(id).await.map_err(map_err)
    }
});

impl_crud_adapter!(TeamAdapter(dyn OrgEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        let teams = self.0.list_teams(DEFAULT_ORG_ID).await.map_err(map_err)?;
        to_json_vec(&teams)
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let team = self.0.get_team(id).await.map_err(map_err)?;
        to_json(&team)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let team: Team = from_json(data)?;
        self.0.create_team(&team).await.map_err(map_err)?;
        to_json(&team)
    }
    async fn update_from_json(&self, _data: Value) -> Result<(), String> {
        Err("Team update not supported via repository interface".to_string())
    }
    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        self.0.delete_team(id).await.map_err(map_err)
    }
});

impl_crud_adapter!(TeamMemberAdapter(dyn OrgEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        Ok(Vec::new())
    }
    async fn list_filtered(
        &self,
        params: &FilterParams,
        valid_sort_fields: &HashSet<String>,
    ) -> Result<FilteredResult, String> {
        let records = match (params.parent_field.as_deref(), params.parent_id.as_deref()) {
            (Some("team_id"), Some(pid)) => {
                let items = self.0.list_team_members(pid).await.map_err(map_err)?;
                to_json_vec(&items)?
            }
            _ => self.list_all().await?,
        };
        Ok(apply_filter_pipeline(records, params, valid_sort_fields))
    }
    async fn get_by_id(&self, _id: &str) -> Result<Value, String> {
        Err("TeamMember get requires team_id context".to_string())
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let member: TeamMember = from_json(data)?;
        self.0.add_team_member(&member).await.map_err(map_err)?;
        to_json(&member)
    }
    async fn update_from_json(&self, _data: Value) -> Result<(), String> {
        Err("TeamMember update not supported".to_string())
    }
    async fn delete_by_id(&self, _id: &str) -> Result<(), String> {
        Err("TeamMember delete requires team_id and user_id".to_string())
    }
});

impl_crud_adapter!(ApiKeyAdapter(dyn OrgEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        let keys = self
            .0
            .list_api_keys(DEFAULT_ORG_ID)
            .await
            .map_err(map_err)?;
        to_json_vec(&keys)
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let key = self.0.get_api_key(id).await.map_err(map_err)?;
        to_json(&key)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let key: ApiKey = from_json(data)?;
        self.0.create_api_key(&key).await.map_err(map_err)?;
        to_json(&key)
    }
    async fn update_from_json(&self, _data: Value) -> Result<(), String> {
        Err("ApiKey update not supported — revoke and recreate".to_string())
    }
    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        self.0.delete_api_key(id).await.map_err(map_err)
    }
});

// ─── Issue group ─────────────────────────────────────────────────────

impl_crud_adapter!(ProjectIssueAdapter(dyn IssueEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        let issues = self
            .0
            .list_issues(DEFAULT_ORG_ID, "")
            .await
            .map_err(map_err)?;
        to_json_vec(&issues)
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let issue = self
            .0
            .get_issue(DEFAULT_ORG_ID, id)
            .await
            .map_err(map_err)?;
        to_json(&issue)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let issue: ProjectIssue = from_json(data)?;
        self.0.create_issue(&issue).await.map_err(map_err)?;
        to_json(&issue)
    }
    async fn update_from_json(&self, data: Value) -> Result<(), String> {
        let issue: ProjectIssue = from_json(data)?;
        self.0.update_issue(&issue).await.map_err(map_err)
    }
    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        self.0
            .delete_issue(DEFAULT_ORG_ID, id)
            .await
            .map_err(map_err)
    }
});

impl_crud_adapter!(IssueCommentAdapter(dyn IssueEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        Ok(Vec::new())
    }
    async fn list_filtered(
        &self,
        params: &FilterParams,
        valid_sort_fields: &HashSet<String>,
    ) -> Result<FilteredResult, String> {
        let records = match (params.parent_field.as_deref(), params.parent_id.as_deref()) {
            (Some("issue_id"), Some(pid)) => {
                let items = self.0.list_comments_by_issue(pid).await.map_err(map_err)?;
                to_json_vec(&items)?
            }
            _ => self.list_all().await?,
        };
        Ok(apply_filter_pipeline(records, params, valid_sort_fields))
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let c = self.0.get_comment(id).await.map_err(map_err)?;
        to_json(&c)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let c: IssueComment = from_json(data)?;
        self.0.create_comment(&c).await.map_err(map_err)?;
        to_json(&c)
    }
    async fn update_from_json(&self, _data: Value) -> Result<(), String> {
        Err("IssueComment update not supported".to_string())
    }
    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        self.0.delete_comment(id).await.map_err(map_err)
    }
});

impl_crud_adapter!(IssueLabelAdapter(dyn IssueEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        let labels = self
            .0
            .list_labels(DEFAULT_ORG_ID, "")
            .await
            .map_err(map_err)?;
        to_json_vec(&labels)
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let l = self.0.get_label(id).await.map_err(map_err)?;
        to_json(&l)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let l: IssueLabel = from_json(data)?;
        self.0.create_label(&l).await.map_err(map_err)?;
        to_json(&l)
    }
    async fn update_from_json(&self, _data: Value) -> Result<(), String> {
        Err("IssueLabel update not supported".to_string())
    }
    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        self.0.delete_label(id).await.map_err(map_err)
    }
});

impl_crud_adapter!(IssueLabelAssignmentAdapter(dyn IssueEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        Ok(Vec::new())
    }
    async fn list_filtered(
        &self,
        params: &FilterParams,
        valid_sort_fields: &HashSet<String>,
    ) -> Result<FilteredResult, String> {
        let records = match (params.parent_field.as_deref(), params.parent_id.as_deref()) {
            (Some("issue_id"), Some(pid)) => {
                let items = self.0.list_labels_for_issue(pid).await.map_err(map_err)?;
                to_json_vec(&items)?
            }
            _ => self.list_all().await?,
        };
        Ok(apply_filter_pipeline(records, params, valid_sort_fields))
    }
    async fn get_by_id(&self, _id: &str) -> Result<Value, String> {
        Err("IssueLabelAssignment has composite key".to_string())
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let a: IssueLabelAssignment = from_json(data)?;
        self.0.assign_label(&a).await.map_err(map_err)?;
        to_json(&a)
    }
    async fn update_from_json(&self, _data: Value) -> Result<(), String> {
        Err("IssueLabelAssignment update not supported".to_string())
    }
    async fn delete_by_id(&self, _id: &str) -> Result<(), String> {
        Err("IssueLabelAssignment delete requires issue_id and label_id".to_string())
    }
});

// ─── Plan group ──────────────────────────────────────────────────────

impl_crud_adapter!(PlanAdapter(dyn PlanEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        let plans = self
            .0
            .list_plans(DEFAULT_ORG_ID, "")
            .await
            .map_err(map_err)?;
        to_json_vec(&plans)
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let plan = self.0.get_plan(DEFAULT_ORG_ID, id).await.map_err(map_err)?;
        to_json(&plan)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let plan: Plan = from_json(data)?;
        self.0.create_plan(&plan).await.map_err(map_err)?;
        to_json(&plan)
    }
    async fn update_from_json(&self, data: Value) -> Result<(), String> {
        let plan: Plan = from_json(data)?;
        self.0.update_plan(&plan).await.map_err(map_err)
    }
    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        self.0
            .delete_plan(DEFAULT_ORG_ID, id)
            .await
            .map_err(map_err)
    }
});

impl_crud_adapter!(PlanVersionAdapter(dyn PlanEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        Ok(Vec::new())
    }
    async fn list_filtered(
        &self,
        params: &FilterParams,
        valid_sort_fields: &HashSet<String>,
    ) -> Result<FilteredResult, String> {
        let records = match (params.parent_field.as_deref(), params.parent_id.as_deref()) {
            (Some("plan_id"), Some(pid)) => {
                let items = self
                    .0
                    .list_plan_versions_by_plan(pid)
                    .await
                    .map_err(map_err)?;
                to_json_vec(&items)?
            }
            _ => self.list_all().await?,
        };
        Ok(apply_filter_pipeline(records, params, valid_sort_fields))
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let v = self.0.get_plan_version(id).await.map_err(map_err)?;
        to_json(&v)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let v: PlanVersion = from_json(data)?;
        self.0.create_plan_version(&v).await.map_err(map_err)?;
        to_json(&v)
    }
    async fn update_from_json(&self, _data: Value) -> Result<(), String> {
        Err("PlanVersion is immutable".to_string())
    }
    async fn delete_by_id(&self, _id: &str) -> Result<(), String> {
        Err("PlanVersion delete not supported".to_string())
    }
});

impl_crud_adapter!(PlanReviewAdapter(dyn PlanEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        Ok(Vec::new())
    }
    async fn list_filtered(
        &self,
        params: &FilterParams,
        valid_sort_fields: &HashSet<String>,
    ) -> Result<FilteredResult, String> {
        let records = match (params.parent_field.as_deref(), params.parent_id.as_deref()) {
            (Some("plan_version_id"), Some(pid)) => {
                let items = self
                    .0
                    .list_plan_reviews_by_version(pid)
                    .await
                    .map_err(map_err)?;
                to_json_vec(&items)?
            }
            _ => self.list_all().await?,
        };
        Ok(apply_filter_pipeline(records, params, valid_sort_fields))
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let r = self.0.get_plan_review(id).await.map_err(map_err)?;
        to_json(&r)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let r: PlanReview = from_json(data)?;
        self.0.create_plan_review(&r).await.map_err(map_err)?;
        to_json(&r)
    }
    async fn update_from_json(&self, _data: Value) -> Result<(), String> {
        Err("PlanReview is immutable".to_string())
    }
    async fn delete_by_id(&self, _id: &str) -> Result<(), String> {
        Err("PlanReview delete not supported".to_string())
    }
});

// ─── VCS group ───────────────────────────────────────────────────────

impl_crud_adapter!(RepositoryAdapter(dyn VcsEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        let repos = self
            .0
            .list_repositories(DEFAULT_ORG_ID, "")
            .await
            .map_err(map_err)?;
        to_json_vec(&repos)
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let r = self
            .0
            .get_repository(DEFAULT_ORG_ID, id)
            .await
            .map_err(map_err)?;
        to_json(&r)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let r: Repository = from_json(data)?;
        self.0.create_repository(&r).await.map_err(map_err)?;
        to_json(&r)
    }
    async fn update_from_json(&self, data: Value) -> Result<(), String> {
        let r: Repository = from_json(data)?;
        self.0.update_repository(&r).await.map_err(map_err)
    }
    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        self.0
            .delete_repository(DEFAULT_ORG_ID, id)
            .await
            .map_err(map_err)
    }
});

impl_crud_adapter!(BranchAdapter(dyn VcsEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        Ok(Vec::new())
    }
    async fn list_filtered(
        &self,
        params: &FilterParams,
        valid_sort_fields: &HashSet<String>,
    ) -> Result<FilteredResult, String> {
        let records = match (params.parent_field.as_deref(), params.parent_id.as_deref()) {
            (Some("repository_id"), Some(pid)) => {
                let items = self.0.list_branches(pid).await.map_err(map_err)?;
                to_json_vec(&items)?
            }
            _ => self.list_all().await?,
        };
        Ok(apply_filter_pipeline(records, params, valid_sort_fields))
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let b = self.0.get_branch(id).await.map_err(map_err)?;
        to_json(&b)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let b: Branch = from_json(data)?;
        self.0.create_branch(&b).await.map_err(map_err)?;
        to_json(&b)
    }
    async fn update_from_json(&self, data: Value) -> Result<(), String> {
        let b: Branch = from_json(data)?;
        self.0.update_branch(&b).await.map_err(map_err)
    }
    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        self.0.delete_branch(id).await.map_err(map_err)
    }
});

impl_crud_adapter!(WorktreeAdapter(dyn VcsEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        Ok(Vec::new())
    }
    async fn list_filtered(
        &self,
        params: &FilterParams,
        valid_sort_fields: &HashSet<String>,
    ) -> Result<FilteredResult, String> {
        let records = match (params.parent_field.as_deref(), params.parent_id.as_deref()) {
            (Some("repository_id"), Some(pid)) => {
                let items = self.0.list_worktrees(pid).await.map_err(map_err)?;
                to_json_vec(&items)?
            }
            _ => self.list_all().await?,
        };
        Ok(apply_filter_pipeline(records, params, valid_sort_fields))
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let w = self.0.get_worktree(id).await.map_err(map_err)?;
        to_json(&w)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let w: Worktree = from_json(data)?;
        self.0.create_worktree(&w).await.map_err(map_err)?;
        to_json(&w)
    }
    async fn update_from_json(&self, data: Value) -> Result<(), String> {
        let w: Worktree = from_json(data)?;
        self.0.update_worktree(&w).await.map_err(map_err)
    }
    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        self.0.delete_worktree(id).await.map_err(map_err)
    }
});

impl_crud_adapter!(AgentWorktreeAssignmentAdapter(dyn VcsEntityRepository) {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        Ok(Vec::new())
    }
    async fn list_filtered(
        &self,
        params: &FilterParams,
        valid_sort_fields: &HashSet<String>,
    ) -> Result<FilteredResult, String> {
        let records = match (params.parent_field.as_deref(), params.parent_id.as_deref()) {
            (Some("worktree_id"), Some(pid)) => {
                let items = self
                    .0
                    .list_assignments_by_worktree(pid)
                    .await
                    .map_err(map_err)?;
                to_json_vec(&items)?
            }
            _ => self.list_all().await?,
        };
        Ok(apply_filter_pipeline(records, params, valid_sort_fields))
    }
    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        let a = self.0.get_assignment(id).await.map_err(map_err)?;
        to_json(&a)
    }
    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let a: AgentWorktreeAssignment = from_json(data)?;
        self.0.create_assignment(&a).await.map_err(map_err)?;
        to_json(&a)
    }
    async fn update_from_json(&self, _data: Value) -> Result<(), String> {
        Err("AgentWorktreeAssignment update not supported — release instead".to_string())
    }
    async fn delete_by_id(&self, _id: &str) -> Result<(), String> {
        Err("AgentWorktreeAssignment delete not supported — release instead".to_string())
    }
});
