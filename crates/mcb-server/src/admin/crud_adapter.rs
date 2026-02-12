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
use rmcp::model::{CallToolRequestParams, Content};
use serde_json::Value;

use crate::args::{EntityAction, EntityArgs, EntityResource};
use crate::tools::{ToolHandlers, route_tool_call};

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

#[derive(Clone)]
struct UnifiedEntityCrudAdapter {
    resource: EntityResource,
    parent_field: Option<&'static str>,
    handlers: ToolHandlers,
}

fn slug_to_resource(slug: &str) -> Option<(EntityResource, Option<&'static str>)> {
    match slug {
        "organizations" => Some((EntityResource::Org, None)),
        "users" => Some((EntityResource::User, None)),
        "teams" => Some((EntityResource::Team, None)),
        "team-members" => Some((EntityResource::TeamMember, Some("team_id"))),
        "api-keys" => Some((EntityResource::ApiKey, None)),
        "project-issues" => Some((EntityResource::Issue, None)),
        "issue-comments" => Some((EntityResource::Comment, Some("issue_id"))),
        "issue-labels" => Some((EntityResource::Label, None)),
        "issue-label-assignments" => Some((EntityResource::LabelAssignment, Some("issue_id"))),
        "plans" => Some((EntityResource::Plan, None)),
        "plan-versions" => Some((EntityResource::Version, Some("plan_id"))),
        "plan-reviews" => Some((EntityResource::Review, Some("plan_version_id"))),
        "repositories" => Some((EntityResource::Repository, None)),
        "branches" => Some((EntityResource::Branch, Some("repository_id"))),
        "worktrees" => Some((EntityResource::Worktree, Some("repository_id"))),
        "agent-worktree-assignments" => Some((EntityResource::Assignment, Some("worktree_id"))),
        _ => None,
    }
}

fn extract_text_content(content: &[Content]) -> String {
    content
        .iter()
        .filter_map(|c| {
            if let Ok(v) = serde_json::to_value(c) {
                v.get("text").and_then(|t| t.as_str()).map(str::to_string)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn base_entity_args(resource: EntityResource, action: EntityAction) -> EntityArgs {
    EntityArgs {
        action,
        resource,
        data: None,
        id: None,
        org_id: None,
        project_id: None,
        repository_id: None,
        worktree_id: None,
        plan_id: None,
        plan_version_id: None,
        issue_id: None,
        label_id: None,
        team_id: None,
        user_id: None,
        email: None,
    }
}

fn apply_parent_scope(args: &mut EntityArgs, parent_field: Option<&str>, parent_id: Option<&str>) {
    let Some(field) = parent_field else {
        return;
    };
    let Some(id) = parent_id else {
        return;
    };

    match field {
        "team_id" => args.team_id = Some(id.to_string()),
        "issue_id" => args.issue_id = Some(id.to_string()),
        "plan_id" => args.plan_id = Some(id.to_string()),
        "plan_version_id" => args.plan_version_id = Some(id.to_string()),
        "repository_id" => args.repository_id = Some(id.to_string()),
        "worktree_id" => args.worktree_id = Some(id.to_string()),
        _ => {}
    }
}

impl UnifiedEntityCrudAdapter {
    fn build_entity_arguments(args: EntityArgs) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "action".to_string(),
            serde_json::Value::String(
                match args.action {
                    EntityAction::Create => "create",
                    EntityAction::Get => "get",
                    EntityAction::Update => "update",
                    EntityAction::List => "list",
                    EntityAction::Delete => "delete",
                    EntityAction::Release => "release",
                }
                .to_string(),
            ),
        );
        map.insert(
            "resource".to_string(),
            serde_json::Value::String(
                match args.resource {
                    EntityResource::Repository => "repository",
                    EntityResource::Branch => "branch",
                    EntityResource::Worktree => "worktree",
                    EntityResource::Assignment => "assignment",
                    EntityResource::Plan => "plan",
                    EntityResource::Version => "version",
                    EntityResource::Review => "review",
                    EntityResource::Issue => "issue",
                    EntityResource::Comment => "comment",
                    EntityResource::Label => "label",
                    EntityResource::LabelAssignment => "label_assignment",
                    EntityResource::Org => "org",
                    EntityResource::User => "user",
                    EntityResource::Team => "team",
                    EntityResource::TeamMember => "team_member",
                    EntityResource::ApiKey => "api_key",
                }
                .to_string(),
            ),
        );

        if let Some(value) = args.data {
            map.insert("data".to_string(), value);
        }
        if let Some(value) = args.id {
            map.insert("id".to_string(), serde_json::Value::String(value));
        }
        if let Some(value) = args.org_id {
            map.insert("org_id".to_string(), serde_json::Value::String(value));
        }
        if let Some(value) = args.project_id {
            map.insert("project_id".to_string(), serde_json::Value::String(value));
        }
        if let Some(value) = args.repository_id {
            map.insert(
                "repository_id".to_string(),
                serde_json::Value::String(value),
            );
        }
        if let Some(value) = args.worktree_id {
            map.insert("worktree_id".to_string(), serde_json::Value::String(value));
        }
        if let Some(value) = args.plan_id {
            map.insert("plan_id".to_string(), serde_json::Value::String(value));
        }
        if let Some(value) = args.plan_version_id {
            map.insert(
                "plan_version_id".to_string(),
                serde_json::Value::String(value),
            );
        }
        if let Some(value) = args.issue_id {
            map.insert("issue_id".to_string(), serde_json::Value::String(value));
        }
        if let Some(value) = args.label_id {
            map.insert("label_id".to_string(), serde_json::Value::String(value));
        }
        if let Some(value) = args.team_id {
            map.insert("team_id".to_string(), serde_json::Value::String(value));
        }
        if let Some(value) = args.user_id {
            map.insert("user_id".to_string(), serde_json::Value::String(value));
        }
        if let Some(value) = args.email {
            map.insert("email".to_string(), serde_json::Value::String(value));
        }

        map
    }

    async fn execute(&self, args: EntityArgs) -> Result<Value, String> {
        let arguments = Self::build_entity_arguments(args);

        let request = CallToolRequestParams {
            name: "entity".into(),
            arguments: Some(arguments),
            task: None,
            meta: None,
        };

        let result = route_tool_call(request, &self.handlers)
            .await
            .map_err(|e| format!("entity dispatch failed: {}", e.message))?;

        let text = extract_text_content(&result.content);
        if result.is_error.unwrap_or(false) {
            return Err(if text.is_empty() {
                "entity operation failed".to_string()
            } else {
                text
            });
        }

        if text.trim().is_empty() {
            Ok(Value::Null)
        } else {
            match serde_json::from_str(&text) {
                Ok(json) => Ok(json),
                Err(_) => Ok(Value::String(text)),
            }
        }
    }

    async fn list_with_parent(&self, parent_id: Option<&str>) -> Result<Vec<Value>, String> {
        let mut args = base_entity_args(self.resource, EntityAction::List);
        apply_parent_scope(&mut args, self.parent_field, parent_id);
        match self.execute(args).await? {
            Value::Array(items) => Ok(items),
            Value::Null => Ok(Vec::new()),
            other => Err(format!("expected list response, got: {other}")),
        }
    }
}

#[async_trait]
impl EntityCrudAdapter for UnifiedEntityCrudAdapter {
    async fn list_all(&self) -> Result<Vec<Value>, String> {
        if self.parent_field.is_some() {
            return Ok(Vec::new());
        }
        self.list_with_parent(None).await
    }

    async fn list_filtered(
        &self,
        params: &FilterParams,
        valid_sort_fields: &HashSet<String>,
    ) -> Result<FilteredResult, String> {
        let records = match (
            self.parent_field,
            params.parent_field.as_deref(),
            params.parent_id.as_deref(),
        ) {
            (Some(expected), Some(actual), Some(parent_id)) if expected == actual => {
                self.list_with_parent(Some(parent_id)).await?
            }
            (Some(_), _, _) => Vec::new(),
            (None, _, _) => self.list_all().await?,
        };
        Ok(apply_filter_pipeline(records, params, valid_sort_fields))
    }

    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        if matches!(
            self.resource,
            EntityResource::TeamMember | EntityResource::LabelAssignment
        ) {
            return Err("resource has a composite key and cannot be fetched by id".to_string());
        }

        let mut args = base_entity_args(self.resource, EntityAction::Get);
        args.id = Some(id.to_string());
        self.execute(args).await
    }

    async fn create_from_json(&self, data: Value) -> Result<Value, String> {
        let mut args = base_entity_args(self.resource, EntityAction::Create);
        args.data = Some(data);
        self.execute(args).await
    }

    async fn update_from_json(&self, data: Value) -> Result<(), String> {
        let mut args = base_entity_args(self.resource, EntityAction::Update);
        args.data = Some(data);
        let _ = self.execute(args).await?;
        Ok(())
    }

    async fn delete_by_id(&self, id: &str) -> Result<(), String> {
        if matches!(
            self.resource,
            EntityResource::TeamMember | EntityResource::LabelAssignment
        ) {
            return Err("resource has a composite key and cannot be deleted by id".to_string());
        }

        let mut args = base_entity_args(self.resource, EntityAction::Delete);
        args.id = Some(id.to_string());
        let _ = self.execute(args).await?;
        Ok(())
    }
}

/// Resolves a CRUD adapter for the given entity slug from AdminState.
pub fn resolve_adapter(slug: &str, state: &AdminState) -> Option<Box<dyn EntityCrudAdapter>> {
    if let Some(handlers) = state.tool_handlers.as_ref()
        && let Some((resource, parent_field)) = slug_to_resource(slug)
    {
        return Some(Box::new(UnifiedEntityCrudAdapter {
            resource,
            parent_field,
            handlers: handlers.clone(),
        }));
    }

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

struct OrgAdapter(Arc<dyn OrgEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for OrgAdapter {
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
}

struct UserAdapter(Arc<dyn OrgEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for UserAdapter {
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
}

struct TeamAdapter(Arc<dyn OrgEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for TeamAdapter {
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
}

struct TeamMemberAdapter(Arc<dyn OrgEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for TeamMemberAdapter {
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
}

struct ApiKeyAdapter(Arc<dyn OrgEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for ApiKeyAdapter {
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
}

// ─── Issue group ─────────────────────────────────────────────────────

struct ProjectIssueAdapter(Arc<dyn IssueEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for ProjectIssueAdapter {
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
}

struct IssueCommentAdapter(Arc<dyn IssueEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for IssueCommentAdapter {
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
}

struct IssueLabelAdapter(Arc<dyn IssueEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for IssueLabelAdapter {
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
}

struct IssueLabelAssignmentAdapter(Arc<dyn IssueEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for IssueLabelAssignmentAdapter {
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
}

// ─── Plan group ──────────────────────────────────────────────────────

struct PlanAdapter(Arc<dyn PlanEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for PlanAdapter {
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
}

struct PlanVersionAdapter(Arc<dyn PlanEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for PlanVersionAdapter {
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
}

struct PlanReviewAdapter(Arc<dyn PlanEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for PlanReviewAdapter {
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
}

// ─── VCS group ───────────────────────────────────────────────────────

struct RepositoryAdapter(Arc<dyn VcsEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for RepositoryAdapter {
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
}

struct BranchAdapter(Arc<dyn VcsEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for BranchAdapter {
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
}

struct WorktreeAdapter(Arc<dyn VcsEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for WorktreeAdapter {
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
}

struct AgentWorktreeAssignmentAdapter(Arc<dyn VcsEntityRepository>);

#[async_trait]
impl EntityCrudAdapter for AgentWorktreeAssignmentAdapter {
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
}
