//! Admin entity registry — schema-driven metadata for auto-generated CRUD UI.
//!
//! Reads `schemars::JsonSchema` derives from domain entities at runtime via
//! `schema_for!()` to produce field metadata consumed by Handlebars templates.

use std::borrow::Cow;

use mcb_domain::entities::{
    AgentWorktreeAssignment, ApiKey, Branch, IssueComment, IssueLabel, IssueLabelAssignment,
    Organization, Plan, PlanReview, PlanVersion, ProjectIssue, Repository, Team, TeamMember, User,
    Worktree,
};
use schemars::{JsonSchema, schema_for};
use serde::Serialize;
use serde_json::Value;

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
    match serde_json::to_value(&schema_for!(T)) {
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
                    AdminFieldMeta {
                        name: name.clone(),
                        label: to_title(name),
                        readonly: is_readonly_field(name),
                        hidden: is_hidden_field(name),
                        is_textarea: input_type.as_ref() == "textarea",
                        is_checkbox: input_type.as_ref() == "checkbox",
                        is_select: input_type.as_ref() == "select",
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
                if let Some(format) = field_schema.get("format").and_then(Value::as_str) {
                    if format == "date-time" {
                        return Cow::Borrowed("datetime-local");
                    }
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
    fn test_registry_has_16_entities() {
        assert_eq!(AdminRegistry::all().len(), 16);
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
        assert!(
            status.enum_values.contains(&"Draft".to_string()),
            "PlanStatus should contain Draft"
        );
        assert!(
            status.enum_values.contains(&"Active".to_string()),
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
