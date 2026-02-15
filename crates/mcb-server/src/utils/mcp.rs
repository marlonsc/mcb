//! Shared helper functions for tool handlers.

use mcb_domain::entities::memory::OriginContext;
use mcb_domain::error::Error;
use mcb_domain::utils::id as domain_id;
use mcb_domain::utils::time as domain_time;
use mcb_domain::value_objects::OrgContext;
use rmcp::model::{CallToolResult, Content, ErrorData as McpError};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use super::json::json_map;
use crate::error_mapping::safe_internal_error;

/// Returns the required `id` parameter or an MCP invalid params error.
pub fn require_id(id: &Option<String>) -> Result<String, McpError> {
    id.clone()
        .ok_or_else(|| McpError::invalid_params("id required", None))
}

/// Serializes a value into pretty JSON and wraps it in a successful MCP tool result.
pub fn ok_json<T: Serialize>(val: &T) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(val)
        .map_err(|e| safe_internal_error("json serialization", &e))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Wraps plain text in a successful MCP tool result.
pub fn ok_text(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Builds a tool error result with a contextual message.
pub fn tool_error(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(msg)])
}

/// Resolves the organization id, preferring explicit input over the current context default.
#[must_use]
pub fn resolve_org_id(explicit: Option<&str>) -> String {
    if let Some(org_id) = explicit {
        return org_id.to_owned();
    }
    OrgContext::default().id_str().clone()
}

/// Normalizes optional identifier input by trimming whitespace and discarding empty values.
#[must_use]
pub fn normalize_identifier(value: Option<&str>) -> Option<String> {
    let raw = value?;

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

/// Resolves identifier precedence between explicit args and payload fields.
///
/// Returns an error when both values are present but conflict.
pub fn resolve_identifier_precedence(
    field: &str,
    args_value: Option<&str>,
    payload_value: Option<&str>,
) -> Result<Option<String>, McpError> {
    let args_normalized = normalize_identifier(args_value);
    let payload_normalized = normalize_identifier(payload_value);

    if let (Some(arg), Some(payload)) = (&args_normalized, &payload_normalized)
        && arg != payload
    {
        return Err(McpError::invalid_params(
            format!("conflicting {field} between args and data"),
            None,
        ));
    }

    Ok(args_normalized.or(payload_normalized))
}

/// Resolve and require an identifier from args/payload precedence.
///
/// Returns an invalid-params error with `required_message` when both values are absent.
pub fn require_resolved_identifier(
    field: &str,
    args_value: Option<&str>,
    payload_value: Option<&str>,
    required_message: &'static str,
) -> Result<String, McpError> {
    resolve_identifier_precedence(field, args_value, payload_value)?
        .ok_or_else(|| McpError::invalid_params(required_message, None))
}

/// Input parameters for resolving an `OriginContext`.
///
/// All fields default to `None`/`false`. Use struct update syntax with `Default::default()`
/// to only specify non-default fields:
/// ```ignore
/// resolve_origin_context(OriginContextInput {
///     org_id: args.org_id.as_deref(),
///     require_project_id: true,
///     ..Default::default()
/// })
/// ```
#[derive(Default)]
pub struct OriginContextInput<'a> {
    /// The organization ID.
    pub org_id: Option<&'a str>,
    /// Project ID from arguments.
    pub project_id_args: Option<&'a str>,
    /// Project ID from payload.
    pub project_id_payload: Option<&'a str>,
    /// Session ID from arguments.
    pub session_from_args: Option<&'a str>,
    /// Session ID from payload.
    pub session_from_data: Option<&'a str>,
    /// Parent session ID from arguments.
    pub parent_session_from_args: Option<&'a str>,
    /// Parent session ID from payload.
    pub parent_session_from_data: Option<&'a str>,
    /// Execution ID from arguments.
    pub execution_from_args: Option<&'a str>,
    /// Execution ID from payload.
    pub execution_from_data: Option<&'a str>,
    /// Tool name from arguments.
    pub tool_name_args: Option<&'a str>,
    /// Tool name from payload.
    pub tool_name_payload: Option<&'a str>,
    /// Repository ID from arguments.
    pub repo_id_args: Option<&'a str>,
    /// Repository ID from payload.
    pub repo_id_payload: Option<&'a str>,
    /// Repository path from arguments.
    pub repo_path_args: Option<&'a str>,
    /// Repository path from payload.
    pub repo_path_payload: Option<&'a str>,
    /// Worktree ID from arguments.
    pub worktree_id_args: Option<&'a str>,
    /// Worktree ID from payload.
    pub worktree_id_payload: Option<&'a str>,
    /// File path from arguments.
    pub file_path_args: Option<&'a str>,
    /// File path from payload.
    pub file_path_payload: Option<&'a str>,
    /// Branch name from arguments.
    pub branch_args: Option<&'a str>,
    /// Branch name from payload.
    pub branch_payload: Option<&'a str>,
    /// Commit hash from arguments.
    pub commit_args: Option<&'a str>,
    /// Commit hash from payload.
    pub commit_payload: Option<&'a str>,
    /// Operator ID from arguments.
    pub operator_id_args: Option<&'a str>,
    /// Operator ID from payload.
    pub operator_id_payload: Option<&'a str>,
    /// Machine ID from arguments.
    pub machine_id_args: Option<&'a str>,
    /// Machine ID from payload.
    pub machine_id_payload: Option<&'a str>,
    /// Agent program from arguments.
    pub agent_program_args: Option<&'a str>,
    /// Agent program from payload.
    pub agent_program_payload: Option<&'a str>,
    /// Model ID from arguments.
    pub model_id_args: Option<&'a str>,
    /// Model ID from payload.
    pub model_id_payload: Option<&'a str>,
    /// Delegated flag from arguments.
    pub delegated_args: Option<bool>,
    /// Delegated flag from payload.
    pub delegated_payload: Option<bool>,
    /// Whether project ID is required.
    pub require_project_id: bool,
    /// Optional timestamp override.
    pub timestamp: Option<i64>,
}

/// Resolves a pair of args/payload identifiers, returning an error on conflict.
fn resolve_field(
    field: &str,
    args_value: Option<&str>,
    payload_value: Option<&str>,
) -> Result<Option<String>, McpError> {
    resolve_identifier_precedence(field, args_value, payload_value)
}

fn resolve_session_correlation(
    field: &str,
    namespace: &str,
    args_value: Option<&str>,
    payload_value: Option<&str>,
) -> Result<Option<String>, McpError> {
    Ok(resolve_field(field, args_value, payload_value)?
        .as_deref()
        .map(|v| domain_id::correlate_id(namespace, v)))
}

/// Common payload fields extracted from a JSON data map for origin context resolution.
pub struct OriginPayloadFields {
    /// Project ID from payload.
    pub project_id: Option<String>,
    /// Session ID from payload.
    pub session_id: Option<String>,
    /// Parent session ID from payload.
    pub parent_session_id: Option<String>,
    /// Repository ID from payload.
    pub repo_id: Option<String>,
    /// Repository path from payload.
    pub repo_path: Option<String>,
    /// Worktree ID from payload.
    pub worktree_id: Option<String>,
    /// File path from payload.
    pub file_path: Option<String>,
    /// Branch name from payload.
    pub branch: Option<String>,
    /// Commit hash from payload.
    pub commit: Option<String>,
    /// Operator ID from payload.
    pub operator_id: Option<String>,
    /// Machine ID from payload.
    pub machine_id: Option<String>,
    /// Agent program from payload.
    pub agent_program: Option<String>,
    /// Model ID from payload.
    pub model_id: Option<String>,
    /// Delegated flag from payload.
    pub delegated: Option<bool>,
}

impl OriginPayloadFields {
    /// Extracts common origin-related fields from a JSON object.
    #[must_use]
    pub fn extract(data: &Map<String, Value>) -> Self {
        Self {
            project_id: opt_str(data, "project_id"),
            session_id: opt_str(data, "session_id"),
            parent_session_id: opt_str(data, "parent_session_id"),
            repo_id: opt_str(data, "repo_id"),
            repo_path: opt_str(data, "repo_path"),
            worktree_id: opt_str(data, "worktree_id"),
            file_path: opt_str(data, "file_path"),
            branch: opt_str(data, "branch"),
            commit: opt_str(data, "commit"),
            operator_id: opt_str(data, "operator_id"),
            machine_id: opt_str(data, "machine_id"),
            agent_program: opt_str(data, "agent_program"),
            model_id: opt_str(data, "model_id"),
            delegated: opt_bool(data, "delegated"),
        }
    }

    /// Converts extracted fields into an `OriginContextInput` with payload slots populated.
    #[must_use]
    pub fn to_input(&self) -> OriginContextInput<'_> {
        OriginContextInput {
            project_id_payload: self.project_id.as_deref(),
            session_from_data: self.session_id.as_deref(),
            parent_session_from_data: self.parent_session_id.as_deref(),
            repo_id_payload: self.repo_id.as_deref(),
            repo_path_payload: self.repo_path.as_deref(),
            worktree_id_payload: self.worktree_id.as_deref(),
            file_path_payload: self.file_path.as_deref(),
            branch_payload: self.branch.as_deref(),
            commit_payload: self.commit.as_deref(),
            operator_id_payload: self.operator_id.as_deref(),
            machine_id_payload: self.machine_id.as_deref(),
            agent_program_payload: self.agent_program.as_deref(),
            model_id_payload: self.model_id.as_deref(),
            delegated_payload: self.delegated,
            ..Default::default()
        }
    }
}

/// Resolves an `OriginContext` from the provided input, handling precedence between args and payload.
pub fn resolve_origin_context(input: OriginContextInput<'_>) -> Result<OriginContext, McpError> {
    let project_id = resolve_field(
        "project_id",
        input.project_id_args,
        input.project_id_payload,
    )?;
    if input.require_project_id && project_id.is_none() {
        return Err(McpError::invalid_params("project_id is required", None));
    }

    Ok(OriginContext::builder()
        .org_id(Some(resolve_org_id(input.org_id)))
        .project_id(project_id)
        .session_id_correlation(resolve_session_correlation(
            "session_id",
            "session",
            input.session_from_args,
            input.session_from_data,
        )?)
        .parent_session_id_correlation(resolve_session_correlation(
            "parent_session_id",
            "parent_session",
            input.parent_session_from_args,
            input.parent_session_from_data,
        )?)
        .execution_id(resolve_field(
            "execution_id",
            input.execution_from_args,
            input.execution_from_data,
        )?)
        .tool_name(resolve_field(
            "tool_name",
            input.tool_name_args,
            input.tool_name_payload,
        )?)
        .repo_id(resolve_field(
            "repo_id",
            input.repo_id_args,
            input.repo_id_payload,
        )?)
        .repo_path(resolve_field(
            "repo_path",
            input.repo_path_args,
            input.repo_path_payload,
        )?)
        .operator_id(resolve_field(
            "operator_id",
            input.operator_id_args,
            input.operator_id_payload,
        )?)
        .machine_id(resolve_field(
            "machine_id",
            input.machine_id_args,
            input.machine_id_payload,
        )?)
        .agent_program(resolve_field(
            "agent_program",
            input.agent_program_args,
            input.agent_program_payload,
        )?)
        .model_id(resolve_field(
            "model_id",
            input.model_id_args,
            input.model_id_payload,
        )?)
        .delegated(input.delegated_args.or(input.delegated_payload))
        .worktree_id(resolve_field(
            "worktree_id",
            input.worktree_id_args,
            input.worktree_id_payload,
        )?)
        .file_path(resolve_field(
            "file_path",
            input.file_path_args,
            input.file_path_payload,
        )?)
        .branch(resolve_field(
            "branch",
            input.branch_args,
            input.branch_payload,
        )?)
        .commit(resolve_field(
            "commit",
            input.commit_args,
            input.commit_payload,
        )?)
        .timestamp(
            input
                .timestamp
                .or(Some(domain_time::epoch_secs_i64().map_err(|e| {
                    safe_internal_error("resolve timestamp", &e)
                })?)),
        )
        .build())
}

/// Deserializes required request data into the target type.
pub fn require_data<T: DeserializeOwned>(
    data: Option<serde_json::Value>,
    msg: &'static str,
) -> Result<T, McpError> {
    let value = data.ok_or_else(|| McpError::invalid_params(msg, None))?;
    serde_json::from_value(value).map_err(|_| McpError::invalid_params("invalid data", None))
}

/// Maps domain errors to opaque MCP-safe errors.
pub fn map_opaque_error<T>(result: Result<T, Error>) -> Result<T, McpError> {
    result.map_err(crate::error_mapping::to_opaque_mcp_error)
}

/// Requires a JSON object from an optional Value, returning an error if missing.
pub fn require_data_map<'a>(
    data: &'a Option<Value>,
    missing_message: &'static str,
) -> Result<&'a Map<String, Value>, CallToolResult> {
    json_map(data).ok_or_else(|| tool_error(missing_message))
}

/// Requires a string value from a JSON object, returning an error if missing.
pub fn require_str(data: &Map<String, Value>, key: &str) -> Result<String, CallToolResult> {
    data.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| tool_error(format!("Missing required field: {key}")))
}

/// Extracts an optional string value from a JSON object.
pub fn opt_str(data: &Map<String, Value>, key: &str) -> Option<String> {
    data.get(key).and_then(Value::as_str).map(str::to_owned)
}

/// Extracts an optional boolean value from a JSON object.
pub fn opt_bool(data: &Map<String, Value>, key: &str) -> Option<bool> {
    data.get(key).and_then(Value::as_bool)
}

/// Extracts a string array from a JSON object, defaulting to empty if missing.
pub fn str_vec(data: &Map<String, Value>, key: &str) -> Vec<String> {
    data.get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}
