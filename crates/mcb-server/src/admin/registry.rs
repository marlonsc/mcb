//! Admin entity registry — schema-driven metadata for auto-generated CRUD UI.
//!
//! Reads `schemars::JsonSchema` derives from domain entities at runtime via
//! `schema_for!()` to produce field metadata consumed by Handlebars templates.

use std::borrow::Cow;

use mcb_domain::entities::{
    AgentSession, AgentWorktreeAssignment, ApiKey, Branch, Checkpoint, Delegation, IssueComment,
    IssueLabel, IssueLabelAssignment, Organization, Plan, PlanReview, PlanVersion, ProjectIssue,
    Repository, Team, TeamMember, ToolCall, User, Worktree,
};
use schemars::{JsonSchema, schema_for};
use serde::Serialize;
use serde_json::{Value, json};

/// Metadata for a single entity field, derived from JSON Schema properties.
#[derive(Debug, Clone, Serialize)]
pub struct AdminFieldMeta {
    /// Field name matching the struct field (e.g. `created_at`).
    pub name: String,
    /// Human-readable label (e.g. `Created At`).
    pub label: String,
    /// HTML input type: text, number, checkbox, select, textarea, datetime-local.
    pub input_type: String,
    /// True for id, `created_at`, `updated_at`.
    pub readonly: bool,
    /// True for `*_hash`, password, secret fields.
    pub hidden: bool,
    /// Pre-computed: `input_type == "textarea"`.
    pub is_textarea: bool,
    /// Pre-computed: `input_type == "checkbox"`.
    pub is_checkbox: bool,
    /// Pre-computed: `input_type == "select"`.
    pub is_select: bool,
    /// Pre-computed: field holds a Unix-epoch timestamp (`name` ends with `_at`
    /// and the detected input type is numeric).
    pub is_timestamp: bool,
    /// True when this field references another entity (name ends in `_id` with a known target).
    pub is_foreign_key: bool,
    /// Entity slug of the FK target (e.g. `"organizations"` for `org_id`).
    pub fk_target_slug: Option<String>,
    /// True when the field holds a JSON blob (name ends with `_json` or schema type is `object`).
    pub is_json_field: bool,
    /// Enum variant names for `<select>` dropdowns (empty when not an enum).
    pub enum_values: Vec<String>,
}

/// Static metadata for a registered admin entity.
#[derive(Debug, Clone, Copy)]
pub struct AdminEntityMeta {
    /// URL-safe slug used in routes (e.g. "project-issues").
    pub slug: &'static str,
    /// Logical group for sidebar grouping (e.g. "org", "vcs").
    pub group: &'static str,
    /// Human-readable display title (e.g. "Project Issues").
    pub title: &'static str,
    /// Function pointer that returns the JSON Schema for this entity.
    pub schema: fn() -> Value,
}

impl AdminEntityMeta {
    /// Extract visible field metadata from the entity's JSON Schema.
    #[must_use]
    pub fn fields(&self) -> Vec<AdminFieldMeta> {
        extract_fields(&(self.schema)())
    }
}

const ENTITIES: &[AdminEntityMeta] = &[
    AdminEntityMeta {
        slug: "agent-sessions",
        group: "agent",
        title: "Agent Sessions",
        schema: schema_agent_session,
    },
    AdminEntityMeta {
        slug: "delegations",
        group: "agent",
        title: "Delegations",
        schema: schema_delegation,
    },
    AdminEntityMeta {
        slug: "tool-calls",
        group: "agent",
        title: "Tool Calls",
        schema: schema_tool_call,
    },
    AdminEntityMeta {
        slug: "checkpoints",
        group: "agent",
        title: "Checkpoints",
        schema: schema_checkpoint,
    },
    AdminEntityMeta {
        slug: "observations",
        group: "memory",
        title: "Observations",
        schema: schema_observation,
    },
    AdminEntityMeta {
        slug: "session-summaries",
        group: "memory",
        title: "Session Summaries",
        schema: schema_session_summary,
    },
    AdminEntityMeta {
        slug: "projects",
        group: "project",
        title: "Projects",
        schema: schema_project,
    },
    AdminEntityMeta {
        slug: "organizations",
        group: "org",
        title: "Organizations",
        schema: schema_organization,
    },
    AdminEntityMeta {
        slug: "users",
        group: "org",
        title: "Users",
        schema: schema_user,
    },
    AdminEntityMeta {
        slug: "teams",
        group: "org",
        title: "Teams",
        schema: schema_team,
    },
    AdminEntityMeta {
        slug: "team-members",
        group: "org",
        title: "Team Members",
        schema: schema_team_member,
    },
    AdminEntityMeta {
        slug: "api-keys",
        group: "org",
        title: "API Keys",
        schema: schema_api_key,
    },
    AdminEntityMeta {
        slug: "project-issues",
        group: "issue",
        title: "Project Issues",
        schema: schema_project_issue,
    },
    AdminEntityMeta {
        slug: "issue-comments",
        group: "issue",
        title: "Issue Comments",
        schema: schema_issue_comment,
    },
    AdminEntityMeta {
        slug: "issue-labels",
        group: "issue",
        title: "Issue Labels",
        schema: schema_issue_label,
    },
    AdminEntityMeta {
        slug: "issue-label-assignments",
        group: "issue",
        title: "Issue Label Assignments",
        schema: schema_issue_label_assignment,
    },
    AdminEntityMeta {
        slug: "plans",
        group: "plan",
        title: "Plans",
        schema: schema_plan,
    },
    AdminEntityMeta {
        slug: "plan-versions",
        group: "plan",
        title: "Plan Versions",
        schema: schema_plan_version,
    },
    AdminEntityMeta {
        slug: "plan-reviews",
        group: "plan",
        title: "Plan Reviews",
        schema: schema_plan_review,
    },
    AdminEntityMeta {
        slug: "repositories",
        group: "vcs",
        title: "Repositories",
        schema: schema_repository,
    },
    AdminEntityMeta {
        slug: "branches",
        group: "vcs",
        title: "Branches",
        schema: schema_branch,
    },
    AdminEntityMeta {
        slug: "worktrees",
        group: "vcs",
        title: "Worktrees",
        schema: schema_worktree,
    },
    AdminEntityMeta {
        slug: "agent-worktree-assignments",
        group: "vcs",
        title: "Agent Worktree Assignments",
        schema: schema_agent_worktree_assignment,
    },
];

/// Central registry of all admin-managed entities.
pub struct AdminRegistry;

impl AdminRegistry {
    /// Returns all registered entities.
    #[must_use]
    pub fn all() -> &'static [AdminEntityMeta] {
        ENTITIES
    }

    /// Finds an entity by its URL slug.
    #[must_use]
    pub fn find(slug: &str) -> Option<&'static AdminEntityMeta> {
        Self::all().iter().find(|entity| entity.slug == slug)
    }
}

fn schema_organization() -> Value {
    schema_json::<Organization>()
}

fn schema_agent_session() -> Value {
    schema_json::<AgentSession>()
}

fn schema_delegation() -> Value {
    schema_json::<Delegation>()
}

fn schema_tool_call() -> Value {
    schema_json::<ToolCall>()
}

fn schema_checkpoint() -> Value {
    schema_json::<Checkpoint>()
}

fn schema_observation() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "string"},
            "project_id": {"type": "string"},
            "content": {"type": "string"},
            "content_hash": {"type": "string"},
            "tags": {"type": "array", "items": {"type": "string"}},
            "type": {"type": "string"},
            "metadata": {"type": "object"},
            "created_at": {"type": "integer"},
            "embedding_id": {"type": "string"}
        }
    })
}

fn schema_session_summary() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "string"},
            "project_id": {"type": "string"},
            "session_id": {"type": "string"},
            "topics": {"type": "array", "items": {"type": "string"}},
            "decisions": {"type": "array", "items": {"type": "string"}},
            "next_steps": {"type": "array", "items": {"type": "string"}},
            "key_files": {"type": "array", "items": {"type": "string"}},
            "created_at": {"type": "integer"}
        }
    })
}

fn schema_project() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "string"},
            "org_id": {"type": "string"},
            "name": {"type": "string"},
            "path": {"type": "string"},
            "created_at": {"type": "integer"},
            "updated_at": {"type": "integer"}
        }
    })
}

fn schema_user() -> Value {
    schema_json::<User>()
}

fn schema_team() -> Value {
    schema_json::<Team>()
}

fn schema_team_member() -> Value {
    schema_json::<TeamMember>()
}

fn schema_api_key() -> Value {
    schema_json::<ApiKey>()
}

fn schema_project_issue() -> Value {
    schema_json::<ProjectIssue>()
}

fn schema_issue_comment() -> Value {
    schema_json::<IssueComment>()
}

fn schema_issue_label() -> Value {
    schema_json::<IssueLabel>()
}

fn schema_issue_label_assignment() -> Value {
    schema_json::<IssueLabelAssignment>()
}

fn schema_plan() -> Value {
    schema_json::<Plan>()
}

fn schema_plan_version() -> Value {
    schema_json::<PlanVersion>()
}

fn schema_plan_review() -> Value {
    schema_json::<PlanReview>()
}

fn schema_repository() -> Value {
    schema_json::<Repository>()
}

fn schema_branch() -> Value {
    schema_json::<Branch>()
}

fn schema_worktree() -> Value {
    schema_json::<Worktree>()
}

fn schema_agent_worktree_assignment() -> Value {
    schema_json::<AgentWorktreeAssignment>()
}

fn schema_json<T: JsonSchema>() -> Value {
    match serde_json::to_value(schema_for!(T)) {
        Ok(value) => value,
        Err(_) => Value::Null,
    }
}

fn extract_fields(schema: &Value) -> Vec<AdminFieldMeta> {
    let defs = schema.get("$defs");

    let mut fields = schema
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| {
            properties
                .iter()
                .map(|(name, field_schema)| {
                    let resolved = resolve_ref(field_schema, defs);
                    let input_type = detect_input_type(name, &resolved);
                    let enum_values = extract_enum_values(&resolved);
                    let is_timestamp = name.ends_with("_at")
                        && matches!(input_type.as_ref(), "number" | "datetime-local");
                    let fk_target = detect_fk_target(name);
                    let is_json_field = name.ends_with("_json")
                        || matches!(resolved.get("type").and_then(Value::as_str), Some("object"));
                    AdminFieldMeta {
                        name: name.clone(),
                        label: to_title(name),
                        readonly: is_readonly_field(name),
                        hidden: is_hidden_field(name),
                        is_textarea: input_type.as_ref() == "textarea",
                        is_checkbox: input_type.as_ref() == "checkbox",
                        is_select: input_type.as_ref() == "select",
                        is_timestamp,
                        is_foreign_key: fk_target.is_some(),
                        fk_target_slug: fk_target.map(String::from),
                        is_json_field,
                        input_type: input_type.into_owned(),
                        enum_values,
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    fields.sort_by(|a, b| a.name.cmp(&b.name));
    fields
}

fn resolve_ref<'a>(field_schema: &'a Value, defs: Option<&'a Value>) -> Cow<'a, Value> {
    if let Some(ref_path) = field_schema.get("$ref").and_then(Value::as_str) {
        let ref_name = ref_path.strip_prefix("#/$defs/").unwrap_or(ref_path);
        if let Some(resolved) = defs.and_then(|d| d.get(ref_name)) {
            return Cow::Borrowed(resolved);
        }
    }
    Cow::Borrowed(field_schema)
}

fn extract_enum_values(schema: &Value) -> Vec<String> {
    if let Some(one_of) = schema.get("oneOf").and_then(Value::as_array) {
        return one_of
            .iter()
            .filter_map(|variant| variant.get("const").and_then(Value::as_str))
            .map(String::from)
            .collect();
    }
    if let Some(enum_arr) = schema.get("enum").and_then(Value::as_array) {
        return enum_arr
            .iter()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect();
    }
    Vec::new()
}

fn is_hidden_field(name: &str) -> bool {
    name.ends_with("_hash") || name.contains("password") || name.contains("secret")
}

fn is_readonly_field(name: &str) -> bool {
    matches!(name, "id" | "created_at" | "updated_at")
}

fn detect_fk_target(field_name: &str) -> Option<&'static str> {
    match field_name {
        "org_id" => Some("organizations"),
        "user_id" | "created_by" | "assignee" | "author_id" | "reviewer_id" => Some("users"),
        "team_id" => Some("teams"),
        "project_id" => Some("projects"),
        "issue_id" | "parent_issue_id" => Some("project-issues"),
        "label_id" => Some("issue-labels"),
        "plan_id" => Some("plans"),
        "plan_version_id" => Some("plan-versions"),
        "repository_id" => Some("repositories"),
        "branch_id" => Some("branches"),
        "worktree_id" => Some("worktrees"),
        _ => None,
    }
}

fn detect_input_type<'a>(name: &str, field_schema: &'a Value) -> Cow<'a, str> {
    if field_schema.get("enum").is_some() || field_schema.get("oneOf").is_some() {
        return Cow::Borrowed("select");
    }

    if let Some(field_type) = field_schema.get("type").and_then(Value::as_str) {
        return match field_type {
            "boolean" => Cow::Borrowed("checkbox"),
            "integer" | "number" => Cow::Borrowed("number"),
            "array" => Cow::Borrowed("textarea"),
            "object" => Cow::Borrowed("textarea"),
            "string" => {
                if let Some(format) = field_schema.get("format").and_then(Value::as_str)
                    && format == "date-time"
                {
                    return Cow::Borrowed("datetime-local");
                }
                if name.ends_with("_json") {
                    Cow::Borrowed("textarea")
                } else {
                    Cow::Borrowed("text")
                }
            }
            _ => Cow::Borrowed("text"),
        };
    }

    Cow::Borrowed("text")
}

fn to_title(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_23_entities() {
        assert_eq!(AdminRegistry::all().len(), 23);
    }

    #[test]
    fn test_registry_find_known_slug() {
        let entity = AdminRegistry::find("organizations");
        assert!(entity.is_some());
        let entity = entity.unwrap();
        assert_eq!(entity.title, "Organizations");
        assert_eq!(entity.group, "org");
    }

    #[test]
    fn test_registry_includes_agent_entities() {
        assert!(AdminRegistry::find("agent-sessions").is_some());
        assert!(AdminRegistry::find("delegations").is_some());
        assert!(AdminRegistry::find("tool-calls").is_some());
        assert!(AdminRegistry::find("checkpoints").is_some());
    }

    #[test]
    fn test_registry_includes_memory_and_project_entities() {
        assert!(AdminRegistry::find("observations").is_some());
        assert!(AdminRegistry::find("session-summaries").is_some());
        assert!(AdminRegistry::find("projects").is_some());
    }

    #[test]
    fn test_registry_find_unknown_slug_returns_none() {
        assert!(AdminRegistry::find("nonexistent").is_none());
    }

    #[test]
    fn test_entity_fields_returns_nonempty() {
        let entity = AdminRegistry::find("users").unwrap();
        let fields = entity.fields();
        assert!(!fields.is_empty());
        assert!(fields.iter().any(|f| f.name == "id"));
    }

    #[test]
    fn test_id_field_is_readonly() {
        let entity = AdminRegistry::find("organizations").unwrap();
        let fields = entity.fields();
        let id_field = fields.iter().find(|f| f.name == "id").unwrap();
        assert!(id_field.readonly);
    }

    #[test]
    fn test_hidden_fields_detected() {
        let entity = AdminRegistry::find("api-keys").unwrap();
        let fields = entity.fields();
        let hidden = fields.iter().filter(|f| f.hidden).count();
        assert!(hidden > 0, "API keys should have hidden hash fields");
    }

    #[test]
    fn test_to_title_converts_snake_case() {
        assert_eq!(to_title("created_at"), "Created At");
        assert_eq!(to_title("id"), "Id");
        assert_eq!(to_title("settings_json"), "Settings Json");
    }

    #[test]
    fn test_all_entity_schemas_are_valid_json() {
        for entity in AdminRegistry::all() {
            let schema = (entity.schema)();
            assert!(
                schema.get("properties").is_some() || schema.get("type").is_some(),
                "Schema for {} should have properties or type",
                entity.slug
            );
        }
    }

    #[test]
    fn test_field_meta_boolean_flags_consistent() {
        for entity in AdminRegistry::all() {
            for field in entity.fields() {
                if field.is_textarea {
                    assert_eq!(
                        field.input_type, "textarea",
                        "{}.{}",
                        entity.slug, field.name
                    );
                }
                if field.is_checkbox {
                    assert_eq!(
                        field.input_type, "checkbox",
                        "{}.{}",
                        entity.slug, field.name
                    );
                }
                if field.is_select {
                    assert_eq!(field.input_type, "select", "{}.{}", entity.slug, field.name);
                }
            }
        }
    }

    #[test]
    fn test_plan_status_field_has_enum_values() {
        let entity = AdminRegistry::find("plans").unwrap();
        let fields = entity.fields();
        let status = fields.iter().find(|f| f.name == "status").unwrap();
        assert!(status.is_select, "status should be a select field");
        assert!(
            !status.enum_values.is_empty(),
            "status should have enum values"
        );
        let enum_values_lower = status
            .enum_values
            .iter()
            .map(|value| value.to_lowercase())
            .collect::<Vec<_>>();
        assert!(
            enum_values_lower.contains(&"draft".to_string()),
            "PlanStatus should contain Draft"
        );
        assert!(
            enum_values_lower.contains(&"active".to_string()),
            "PlanStatus should contain Active"
        );
    }

    #[test]
    fn test_enum_values_empty_for_string_fields() {
        let entity = AdminRegistry::find("organizations").unwrap();
        let fields = entity.fields();
        let name_field = fields.iter().find(|f| f.name == "name").unwrap();
        assert!(
            name_field.enum_values.is_empty(),
            "string fields should have no enum values"
        );
    }

    #[test]
    fn test_timestamp_fields_detected() {
        let entity = AdminRegistry::find("organizations").unwrap();
        let fields = entity.fields();
        let created_at = fields.iter().find(|f| f.name == "created_at").unwrap();
        assert!(
            created_at.is_timestamp,
            "created_at should be detected as timestamp"
        );
        let name = fields.iter().find(|f| f.name == "name").unwrap();
        assert!(!name.is_timestamp, "name should not be a timestamp");
    }

    #[test]
    fn test_fk_detection_known_fields() {
        assert_eq!(detect_fk_target("org_id"), Some("organizations"));
        assert_eq!(detect_fk_target("user_id"), Some("users"));
        assert_eq!(detect_fk_target("created_by"), Some("users"));
        assert_eq!(detect_fk_target("assignee"), Some("users"));
        assert_eq!(detect_fk_target("author_id"), Some("users"));
        assert_eq!(detect_fk_target("reviewer_id"), Some("users"));
        assert_eq!(detect_fk_target("team_id"), Some("teams"));
        assert_eq!(detect_fk_target("project_id"), Some("projects"));
        assert_eq!(detect_fk_target("issue_id"), Some("project-issues"));
        assert_eq!(detect_fk_target("parent_issue_id"), Some("project-issues"));
        assert_eq!(detect_fk_target("label_id"), Some("issue-labels"));
        assert_eq!(detect_fk_target("plan_id"), Some("plans"));
        assert_eq!(detect_fk_target("plan_version_id"), Some("plan-versions"));
        assert_eq!(detect_fk_target("repository_id"), Some("repositories"));
        assert_eq!(detect_fk_target("branch_id"), Some("branches"));
        assert_eq!(detect_fk_target("worktree_id"), Some("worktrees"));
    }

    #[test]
    fn test_fk_detection_unknown_fields() {
        assert_eq!(detect_fk_target("name"), None);
        assert_eq!(detect_fk_target("id"), None);
        assert_eq!(detect_fk_target("status"), None);
        assert_eq!(detect_fk_target("created_at"), None);
    }

    #[test]
    fn test_fk_fields_detected_in_entities() {
        let entity = AdminRegistry::find("team-members").unwrap();
        let fields = entity.fields();
        let team_id = fields.iter().find(|f| f.name == "team_id").unwrap();
        assert!(team_id.is_foreign_key);
        assert_eq!(team_id.fk_target_slug.as_deref(), Some("teams"));

        let user_id = fields.iter().find(|f| f.name == "user_id").unwrap();
        assert!(user_id.is_foreign_key);
        assert_eq!(user_id.fk_target_slug.as_deref(), Some("users"));
    }

    #[test]
    fn test_non_fk_fields_not_flagged() {
        let entity = AdminRegistry::find("organizations").unwrap();
        let fields = entity.fields();
        let name = fields.iter().find(|f| f.name == "name").unwrap();
        assert!(!name.is_foreign_key);
        assert!(name.fk_target_slug.is_none());
    }

    #[test]
    fn test_json_field_detection() {
        for entity in AdminRegistry::all() {
            for field in entity.fields() {
                if field.name.ends_with("_json") {
                    assert!(
                        field.is_json_field,
                        "{}.{} should be detected as json field",
                        entity.slug, field.name
                    );
                }
            }
        }
    }

    #[test]
    fn test_select_fields_always_have_enum_values() {
        for entity in AdminRegistry::all() {
            for field in entity.fields() {
                if field.is_select {
                    assert!(
                        !field.enum_values.is_empty(),
                        "{}.{} is select but has no enum values",
                        entity.slug,
                        field.name
                    );
                }
            }
        }
    }
}
