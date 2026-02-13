use std::time::{SystemTime, UNIX_EPOCH};

use mcb_domain::entities::memory::OriginContext;
use mcb_domain::error::Error;
use mcb_domain::utils::compute_stable_id_hash;
use mcb_domain::value_objects::OrgContext;
use rmcp::model::{CallToolResult, Content, ErrorData as McpError};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Returns the current Unix timestamp in seconds.
pub fn current_timestamp() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => i64::try_from(duration.as_secs()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

/// Returns the required `id` parameter or an MCP invalid params error.
pub fn require_id(id: &Option<String>) -> Result<String, McpError> {
    id.clone()
        .ok_or_else(|| McpError::invalid_params("id required", None))
}

/// Serializes a value into pretty JSON and wraps it in a successful MCP tool result.
pub fn ok_json<T: Serialize>(val: &T) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(val)
        .map_err(|_| McpError::internal_error("serialization failed", None))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Wraps plain text in a successful MCP tool result.
pub fn ok_text(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Resolves the organization id, preferring explicit input over the current context default.
pub fn resolve_org_id(explicit: Option<&str>) -> String {
    if let Some(org_id) = explicit {
        return org_id.to_string();
    }
    OrgContext::current().id_str().to_string()
}

/// Normalizes optional identifier input by trimming whitespace and discarding empty values.
pub fn normalize_identifier(value: Option<&str>) -> Option<String> {
    let raw = value?;

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
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

/// Input parameters for resolving an `OriginContext`.
#[allow(missing_docs)]
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

/// Resolves an `OriginContext` from the provided input, handling precedence between args and payload.
#[allow(missing_docs)]
pub fn resolve_origin_context(input: OriginContextInput<'_>) -> Result<OriginContext, McpError> {
    let project_id = resolve_identifier_precedence(
        "project_id",
        input.project_id_args,
        input.project_id_payload,
    )?;
    if input.require_project_id && project_id.is_none() {
        return Err(McpError::invalid_params("project_id is required", None));
    }

    let session_id = resolve_identifier_precedence(
        "session_id",
        input.session_from_args,
        input.session_from_data,
    )?;
    let parent_session_id = resolve_identifier_precedence(
        "parent_session_id",
        input.parent_session_from_args,
        input.parent_session_from_data,
    )?;

    let session_id_hash = session_id
        .as_deref()
        .map(|value| compute_stable_id_hash("session", value));
    let parent_session_id_hash = parent_session_id
        .as_deref()
        .map(|value| compute_stable_id_hash("parent_session", value));

    Ok(OriginContext {
        org_id: Some(resolve_org_id(input.org_id)),
        project_id,
        session_id,
        session_id_hash,
        parent_session_id,
        parent_session_id_hash,
        execution_id: resolve_identifier_precedence(
            "execution_id",
            input.execution_from_args,
            input.execution_from_data,
        )?,
        tool_name: resolve_identifier_precedence(
            "tool_name",
            input.tool_name_args,
            input.tool_name_payload,
        )?,
        repo_id: resolve_identifier_precedence(
            "repo_id",
            input.repo_id_args,
            input.repo_id_payload,
        )?,
        repo_path: resolve_identifier_precedence(
            "repo_path",
            input.repo_path_args,
            input.repo_path_payload,
        )?,
        operator_id: resolve_identifier_precedence(
            "operator_id",
            input.operator_id_args,
            input.operator_id_payload,
        )?,
        machine_id: resolve_identifier_precedence(
            "machine_id",
            input.machine_id_args,
            input.machine_id_payload,
        )?,
        agent_program: resolve_identifier_precedence(
            "agent_program",
            input.agent_program_args,
            input.agent_program_payload,
        )?,
        model_id: resolve_identifier_precedence(
            "model_id",
            input.model_id_args,
            input.model_id_payload,
        )?,
        delegated: input.delegated_args.or(input.delegated_payload),
        worktree_id: resolve_identifier_precedence(
            "worktree_id",
            input.worktree_id_args,
            input.worktree_id_payload,
        )?,
        file_path: resolve_identifier_precedence(
            "file_path",
            input.file_path_args,
            input.file_path_payload,
        )?,
        branch: resolve_identifier_precedence("branch", input.branch_args, input.branch_payload)?,
        commit: resolve_identifier_precedence("commit", input.commit_args, input.commit_payload)?,
        timestamp: input.timestamp.or(Some(current_timestamp())),
    })
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
