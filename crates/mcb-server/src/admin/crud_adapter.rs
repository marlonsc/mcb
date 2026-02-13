//! Generic CRUD adapter that bridges entity handlers with domain services.
//!
//! Each entity slug maps to an adapter implementation that knows how to
//! call the correct service methods and serialize results to JSON.

use std::collections::HashSet;

use async_trait::async_trait;
use rmcp::model::{CallToolRequestParams, Content};
use serde_json::Value;

use crate::args::{EntityAction, EntityArgs, EntityResource};
use crate::tools::{ToolExecutionContext, ToolHandlers, route_tool_call};

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
        match self.list_all().await {
            Ok(records) => Ok(apply_filter_pipeline(records, params, valid_sort_fields)),
            Err(e) => Err(e),
        }
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
    fn extract_project_id(data: &Value) -> Option<String> {
        data.get("project_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
    }

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

        let result = route_tool_call(request, &self.handlers, ToolExecutionContext::default())
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
        // If dependent entity, listing all without context is usually invalid or empty in this design,
        // unless we want to allow it. The previous logic returned empty for dependent entities.
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
            // Dependent entity with matching parent context
            (Some(expected), Some(actual), Some(parent_id)) if expected == actual => {
                self.list_with_parent(Some(parent_id)).await?
            }
            // Dependent entity without correct parent context -> empty
            (Some(_), _, _) => Vec::new(),
            // Independent entity -> list all
            (None, _, _) => self.list_all().await?,
        };
        Ok(apply_filter_pipeline(records, params, valid_sort_fields))
    }

    async fn get_by_id(&self, id: &str) -> Result<Value, String> {
        // Validation for composite keys
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
        if matches!(self.resource, EntityResource::Repository) {
            args.project_id = Self::extract_project_id(&data);
            if args.project_id.is_none() {
                return Err("project_id is required for repository create".to_string());
            }
        }
        args.data = Some(data);
        self.execute(args).await
    }

    async fn update_from_json(&self, data: Value) -> Result<(), String> {
        let mut args = base_entity_args(self.resource, EntityAction::Update);
        if matches!(self.resource, EntityResource::Repository) {
            args.project_id = Self::extract_project_id(&data);
            if args.project_id.is_none() {
                return Err("project_id is required for repository update".to_string());
            }
        }
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
        if matches!(self.resource, EntityResource::Repository) {
            return Err("project_id is required for repository delete".to_string());
        }
        args.id = Some(id.to_string());
        let _ = self.execute(args).await?;
        Ok(())
    }
}

/// Resolves a CRUD adapter for the given entity slug from AdminState.
pub fn resolve_adapter(slug: &str, state: &AdminState) -> Option<Box<dyn EntityCrudAdapter>> {
    let handlers = state.tool_handlers.as_ref()?;
    let (resource, parent_field) = slug_to_resource(slug)?;

    Some(Box::new(UnifiedEntityCrudAdapter {
        resource,
        parent_field,
        handlers: handlers.clone(),
    }))
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
