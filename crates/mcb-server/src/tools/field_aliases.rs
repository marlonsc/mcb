//! Field alias resolution for execution context metadata.
//!
//! Provides canonical field name mappings and resolution helpers for
//! extracting execution context values from request and context metadata.

use std::collections::HashMap;

/// Canonical string field → alias mapping for context resolution.
///
/// Each entry maps a canonical key to all accepted aliases (`camelCase`, `x-header`, `snake_case`).
pub const STRING_FIELD_ALIASES: &[(&str, &[&str])] = &[
    (
        "session_id",
        &["session_id", "sessionId", "x-session-id", "x_session_id"],
    ),
    (
        "parent_session_id",
        &[
            "parent_session_id",
            "parentSessionId",
            "x-parent-session-id",
            "x_parent_session_id",
        ],
    ),
    ("org_id", &["org_id", "orgId", "x-org-id", "x_org_id"]),
    (
        "project_id",
        &["project_id", "projectId", "x-project-id", "x_project_id"],
    ),
    (
        "worktree_id",
        &[
            "worktree_id",
            "worktreeId",
            "x-worktree-id",
            "x_worktree_id",
        ],
    ),
    ("repo_id", &["repo_id", "repoId", "x-repo-id", "x_repo_id"]),
    (
        "repo_path",
        &["repo_path", "repoPath", "x-repo-path", "x_repo_path"],
    ),
    (
        "workspace_root",
        &["workspace_root", "workspaceRoot", "x-workspace-root"],
    ),
    (
        "operator_id",
        &[
            "operator_id",
            "operatorId",
            "x-operator-id",
            "x_operator_id",
        ],
    ),
    (
        "machine_id",
        &["machine_id", "machineId", "x-machine-id", "x_machine_id"],
    ),
    (
        "agent_program",
        &[
            "agent_program",
            "agentProgram",
            "ide",
            "x-agent-program",
            "x_agent_program",
        ],
    ),
    (
        "model_id",
        &["model_id", "model", "modelId", "x-model-id", "x_model_id"],
    ),
    (
        "execution_flow",
        &[
            "execution_flow",
            "executionFlow",
            "x-execution-flow",
            "x_execution_flow",
        ],
    ),
];

/// Canonical boolean field → alias mapping for context resolution.
pub const BOOL_FIELD_ALIASES: &[(&str, &[&str])] = &[(
    "delegated",
    &["delegated", "is_delegated", "isDelegated", "x-delegated"],
)];

/// Look up aliases for a canonical field name from the alias tables.
#[must_use]
pub fn field_aliases(canonical: &str) -> &'static [&'static str] {
    STRING_FIELD_ALIASES
        .iter()
        .chain(BOOL_FIELD_ALIASES.iter())
        .find(|&&(k, _)| k == canonical)
        .map_or(&[] as &[&str], |&(_, aliases)| aliases)
}

/// Convert a string to a JSON value.
#[must_use]
pub fn str_value(s: &str) -> serde_json::Value {
    serde_json::Value::String(s.to_owned())
}

/// Insert an override value into the overrides map if present.
pub fn insert_override(overrides: &mut HashMap<String, String>, key: &str, value: Option<String>) {
    if let Some(value) = normalize_text(value) {
        overrides.insert(key.to_owned(), value);
    }
}

/// Resolve an override value from the overrides map by checking all aliases.
#[must_use]
pub fn resolve_override_value(
    overrides: &HashMap<String, String>,
    keys: &[&str],
) -> Option<String> {
    for key in keys {
        if let Some(value) = normalize_text(overrides.get(*key).cloned()) {
            return Some(value);
        }
    }
    None
}

/// Resolve a boolean override value from the overrides map by checking all aliases.
#[must_use]
pub fn resolve_override_bool(overrides: &HashMap<String, String>, keys: &[&str]) -> Option<bool> {
    for key in keys {
        let Some(raw) = overrides.get(*key) else {
            continue;
        };

        match raw.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" => return Some(true),
            "false" | "0" | "no" => return Some(false),
            _ => continue,
        }
    }

    None
}

/// Normalize text by trimming and filtering empty strings.
#[must_use]
pub fn normalize_text(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}
