//! Tool execution context extraction and resolution.
//!
//! Provides the `ToolExecutionContext` struct for capturing execution provenance
//! at the transport boundary and resolving context values from request/context metadata.

use std::collections::HashMap;

use rmcp::model::{CallToolRequestParams, Meta};
use serde_json::Value;

use crate::tools::defaults::RuntimeDefaults;
use crate::tools::field_aliases::{
    BOOL_FIELD_ALIASES, STRING_FIELD_ALIASES, field_aliases, normalize_text, resolve_override_bool,
    resolve_override_value,
};

#[derive(Debug, Clone, Default)]
/// Execution context extracted at transport boundary and propagated to hooks.
pub struct ToolExecutionContext {
    /// Canonical session identifier for the current tool call.
    pub session_id: Option<String>,
    /// Optional parent session identifier for delegated/subagent calls.
    pub parent_session_id: Option<String>,
    /// Optional project identifier associated with this execution.
    pub project_id: Option<String>,
    /// Optional worktree identifier associated with this execution.
    pub worktree_id: Option<String>,
    /// Optional repository identifier associated with this execution.
    pub repo_id: Option<String>,
    /// Optional repository/workspace path associated with this execution.
    pub repo_path: Option<String>,
    /// Optional operator/user identifier for this execution.
    pub operator_id: Option<String>,
    /// Optional machine/host fingerprint for this execution.
    pub machine_id: Option<String>,
    /// Optional agent program/IDE identifier for this execution.
    pub agent_program: Option<String>,
    /// Optional model identifier for this execution.
    pub model_id: Option<String>,
    /// Optional delegated flag for this execution.
    pub delegated: Option<bool>,
    /// Execution timestamp (Unix timestamp in seconds).
    pub timestamp: Option<i64>,
    /// Optional execution flow identifier for tracing.
    pub execution_flow: Option<String>,
}

impl ToolExecutionContext {
    /// Collect request/context metadata into canonical override keys.
    #[must_use]
    pub fn metadata_overrides(
        request_meta: Option<&Meta>,
        context_meta: &Meta,
    ) -> HashMap<String, String> {
        let mut overrides = HashMap::new();
        for &(canonical, aliases) in STRING_FIELD_ALIASES {
            crate::tools::field_aliases::insert_override(
                &mut overrides,
                canonical,
                resolve_context_value(request_meta, context_meta, aliases),
            );
        }
        for &(canonical, aliases) in BOOL_FIELD_ALIASES {
            if let Some(val) = resolve_context_bool(request_meta, context_meta, aliases) {
                overrides.insert(canonical.to_owned(), val.to_string());
            }
        }
        overrides
    }

    /// Resolve execution context from explicit overrides and runtime defaults.
    #[must_use]
    pub fn resolve(defaults: &RuntimeDefaults, overrides: &HashMap<String, String>) -> Self {
        let session_id = resolve_override_value(overrides, field_aliases("session_id"))
            .or_else(|| defaults.session_id.clone());
        let parent_session_id =
            resolve_override_value(overrides, field_aliases("parent_session_id"));
        let project_id = resolve_override_value(overrides, field_aliases("project_id"));
        let worktree_id = resolve_override_value(overrides, field_aliases("worktree_id"));
        let repo_id = resolve_override_value(overrides, field_aliases("repo_id"))
            .or_else(|| defaults.repo_id.clone());
        let repo_path = resolve_override_value(overrides, field_aliases("repo_path"))
            .or_else(|| resolve_override_value(overrides, field_aliases("workspace_root")))
            .or_else(|| defaults.repo_path.clone())
            .or_else(|| defaults.workspace_root.clone());
        let operator_id = resolve_override_value(overrides, field_aliases("operator_id"))
            .or_else(|| defaults.operator_id.clone());
        let machine_id = resolve_override_value(overrides, field_aliases("machine_id"))
            .or_else(|| defaults.machine_id.clone());
        let agent_program = resolve_override_value(overrides, field_aliases("agent_program"))
            .or_else(|| defaults.agent_program.clone());
        let model_id = resolve_override_value(overrides, field_aliases("model_id"))
            .or_else(|| defaults.model_id.clone());
        let delegated = resolve_override_bool(overrides, field_aliases("delegated"))
            .or(Some(parent_session_id.is_some()));
        let execution_flow = resolve_override_value(overrides, field_aliases("execution_flow"))
            .or_else(|| defaults.execution_flow.map(|f| f.to_string()));

        Self {
            session_id,
            parent_session_id,
            project_id,
            worktree_id,
            repo_id,
            repo_path,
            operator_id,
            machine_id,
            agent_program,
            model_id,
            delegated,
            timestamp: mcb_domain::utils::time::epoch_secs_i64().ok(),
            execution_flow,
        }
    }

    /// Inject execution context into tool arguments when those keys are missing.
    pub fn apply_to_request_if_missing(&self, request: &mut CallToolRequestParams) {
        let args = request.arguments.get_or_insert_with(Default::default);
        for (key, value) in [
            (
                "session_id",
                self.session_id
                    .as_deref()
                    .map(crate::tools::field_aliases::str_value),
            ),
            (
                "parent_session_id",
                self.parent_session_id
                    .as_deref()
                    .map(crate::tools::field_aliases::str_value),
            ),
            (
                "project_id",
                self.project_id
                    .as_deref()
                    .map(crate::tools::field_aliases::str_value),
            ),
            (
                "worktree_id",
                self.worktree_id
                    .as_deref()
                    .map(crate::tools::field_aliases::str_value),
            ),
            (
                "repo_id",
                self.repo_id
                    .as_deref()
                    .map(crate::tools::field_aliases::str_value),
            ),
            (
                "repo_path",
                self.repo_path
                    .as_deref()
                    .map(crate::tools::field_aliases::str_value),
            ),
            (
                "operator_id",
                self.operator_id
                    .as_deref()
                    .map(crate::tools::field_aliases::str_value),
            ),
            (
                "machine_id",
                self.machine_id
                    .as_deref()
                    .map(crate::tools::field_aliases::str_value),
            ),
            (
                "agent_program",
                self.agent_program
                    .as_deref()
                    .map(crate::tools::field_aliases::str_value),
            ),
            (
                "model_id",
                self.model_id
                    .as_deref()
                    .map(crate::tools::field_aliases::str_value),
            ),
            ("delegated", self.delegated.map(Value::Bool)),
            ("timestamp", self.timestamp.map(|v| Value::Number(v.into()))),
            (
                "execution_flow",
                self.execution_flow
                    .as_deref()
                    .map(crate::tools::field_aliases::str_value),
            ),
        ] {
            if let Some(v) = value {
                args.entry(key.to_owned()).or_insert(v);
            }
        }
    }
}

/// Extract a string value from metadata by checking all aliases.
fn meta_value_as_string(meta: &Meta, keys: &[&str]) -> Option<String> {
    for key in keys {
        let Some(value) = meta.get(*key) else {
            continue;
        };

        let extracted = match value {
            Value::String(v) => normalize_text(Some(v.clone())),
            Value::Number(v) => Some(v.to_string()),
            Value::Bool(v) => Some(v.to_string()),
            Value::Null | Value::Array(_) | Value::Object(_) => None,
        };

        if extracted.is_some() {
            return extracted;
        }
    }

    None
}

/// Resolve a string value from request or context metadata.
fn resolve_context_value(
    request_meta: Option<&Meta>,
    context_meta: &Meta,
    keys: &[&str],
) -> Option<String> {
    request_meta
        .and_then(|meta| meta_value_as_string(meta, keys))
        .or_else(|| meta_value_as_string(context_meta, keys))
}

/// Extract a boolean value from metadata by checking all aliases.
fn meta_value_as_bool(meta: &Meta, keys: &[&str]) -> Option<bool> {
    for key in keys {
        let Some(value) = meta.get(*key) else {
            continue;
        };

        let extracted = match value {
            Value::Bool(v) => Some(*v),
            Value::String(v) => match v.trim().to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            },
            Value::Null | Value::Number(_) | Value::Array(_) | Value::Object(_) => None,
        };

        if extracted.is_some() {
            return extracted;
        }
    }

    None
}

/// Resolve a boolean value from request or context metadata.
fn resolve_context_bool(
    request_meta: Option<&Meta>,
    context_meta: &Meta,
    keys: &[&str],
) -> Option<bool> {
    request_meta
        .and_then(|meta| meta_value_as_bool(meta, keys))
        .or_else(|| meta_value_as_bool(context_meta, keys))
}
