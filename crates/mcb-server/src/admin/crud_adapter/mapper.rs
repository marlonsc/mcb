//! Mapping utilities between slugs, resources, and args.

use crate::args::{EntityAction, EntityArgs, EntityResource};
use serde_json::Value;

/// Map a URL slug to its entity resource and optional parent field.
#[must_use]
pub fn slug_to_resource(slug: &str) -> Option<(EntityResource, Option<&'static str>)> {
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

/// Build a default `EntityArgs` with only `resource` and `action` set.
#[must_use]
pub fn base_entity_args(resource: EntityResource, action: EntityAction) -> EntityArgs {
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

/// Set the parent scope field on `args` (e.g. `team_id`, `issue_id`).
pub fn apply_parent_scope(
    args: &mut EntityArgs,
    parent_field: Option<&str>,
    parent_id: Option<&str>,
) {
    let Some(field) = parent_field else {
        return;
    };
    let Some(id) = parent_id else {
        return;
    };

    match field {
        "team_id" => args.team_id = Some(id.to_owned()),
        "issue_id" => args.issue_id = Some(id.to_owned()),
        "plan_id" => args.plan_id = Some(id.to_owned()),
        "plan_version_id" => args.plan_version_id = Some(id.to_owned()),
        "repository_id" => args.repository_id = Some(id.to_owned()),
        "worktree_id" => args.worktree_id = Some(id.to_owned()),
        _ => {}
    }
}

/// Extract a non-empty `project_id` string from a JSON value.
pub fn extract_project_id(data: &Value) -> Option<String> {
    data.get("project_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

/// Serialize `EntityArgs` into a JSON map suitable for tool dispatch.
#[must_use]
pub fn build_entity_arguments(args: EntityArgs) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    map.insert(
        "action".to_owned(),
        Value::String(
            match args.action {
                EntityAction::Create => "create",
                EntityAction::Get => "get",
                EntityAction::Update => "update",
                EntityAction::List => "list",
                EntityAction::Delete => "delete",
                EntityAction::Release => "release",
            }
            .to_owned(),
        ),
    );
    map.insert(
        "resource".to_owned(),
        Value::String(
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
            .to_owned(),
        ),
    );

    if let Some(value) = args.data {
        map.insert("data".to_owned(), value);
    }
    if let Some(value) = args.id {
        map.insert("id".to_owned(), Value::String(value));
    }
    if let Some(value) = args.org_id {
        map.insert("org_id".to_owned(), Value::String(value));
    }
    if let Some(value) = args.project_id {
        map.insert("project_id".to_owned(), Value::String(value));
    }
    if let Some(value) = args.repository_id {
        map.insert("repository_id".to_owned(), Value::String(value));
    }
    if let Some(value) = args.worktree_id {
        map.insert("worktree_id".to_owned(), Value::String(value));
    }
    if let Some(value) = args.plan_id {
        map.insert("plan_id".to_owned(), Value::String(value));
    }
    if let Some(value) = args.plan_version_id {
        map.insert("plan_version_id".to_owned(), Value::String(value));
    }
    if let Some(value) = args.issue_id {
        map.insert("issue_id".to_owned(), Value::String(value));
    }
    if let Some(value) = args.label_id {
        map.insert("label_id".to_owned(), Value::String(value));
    }
    if let Some(value) = args.team_id {
        map.insert("team_id".to_owned(), Value::String(value));
    }
    if let Some(value) = args.user_id {
        map.insert("user_id".to_owned(), Value::String(value));
    }
    if let Some(value) = args.email {
        map.insert("email".to_owned(), Value::String(value));
    }

    map
}
