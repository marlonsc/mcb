//! Origin context resolution for MCP tool requests.

use mcb_domain::entities::memory::OriginContext;
use mcb_utils::utils::id as domain_id;
use mcb_utils::utils::time as domain_time;
use rmcp::model::ErrorData as McpError;
use serde_json::{Map, Value};

use super::{opt_bool, opt_str, resolve_identifier_precedence, resolve_org_id};
use crate::error_mapping::safe_internal_error;

/// Resolves a pair of args/payload identifiers, returning an error on conflict.
fn resolve_field(
    field: &str,
    args_value: Option<&str>,
    payload_value: Option<&str>,
) -> Result<Option<String>, McpError> {
    resolve_identifier_precedence(field, args_value, payload_value)
}

/// Resolves a session correlation ID from args/payload precedence.
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

/// Resolves an `OriginContext` from the provided input, handling precedence between args
/// and payload.
///
/// # Errors
/// Returns an error when required fields are missing or conflicting identifiers are provided.
pub fn resolve_origin_context(input: &OriginContextInput<'_>) -> Result<OriginContext, McpError> {
    let project_id = resolve_field(
        "project_id",
        input.project_id_args,
        input.project_id_payload,
    )?;
    if input.require_project_id && project_id.is_none() {
        return Err(McpError::invalid_params("project_id is required", None));
    }

    let session_correlation = resolve_session_correlation(
        "session_id",
        "session",
        input.session_from_args,
        input.session_from_data,
    )?;
    let parent_session_correlation = resolve_session_correlation(
        "parent_session_id",
        "parent_session",
        input.parent_session_from_args,
        input.parent_session_from_data,
    )?;
    let timestamp = match input.timestamp {
        Some(ts) => Some(ts),
        None => Some(
            domain_time::epoch_secs_i64()
                .map_err(|e| safe_internal_error("resolve timestamp", &e))?,
        ),
    };

    build_origin_context(
        input,
        project_id,
        session_correlation,
        parent_session_correlation,
        timestamp,
    )
}

/// Assemble the `OriginContext` from pre-resolved primary fields plus per-field precedence.
fn build_origin_context(
    input: &OriginContextInput<'_>,
    project_id: Option<String>,
    session_correlation: Option<String>,
    parent_session_correlation: Option<String>,
    timestamp: Option<i64>,
) -> Result<OriginContext, McpError> {
    Ok(OriginContext::builder()
        .org_id(Some(resolve_org_id(input.org_id)))
        .project_id(project_id)
        .session_id_correlation(session_correlation)
        .parent_session_id_correlation(parent_session_correlation)
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
        .timestamp(timestamp)
        .build())
}
