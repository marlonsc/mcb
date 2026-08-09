//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use mcb_utils::constants::keys as schema;
use std::sync::Arc;

use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus, AgentType};
use mcb_domain::entities::memory::OriginContext;
use mcb_domain::ports::AgentSessionServiceInterface;
use mcb_utils::utils::id as domain_id;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde::Deserialize;

use super::common::{parse_agent_type, require_data_map, require_str};
use crate::args::SessionArgs;
use crate::error_mapping::{safe_internal_error, to_contextual_tool_error};
use crate::formatter::ResponseFormatter;
use crate::utils::mcp::{
    OriginContextInput, resolve_identifier_precedence, resolve_origin_context, tool_error,
};

/// Payload for creating an agent session from JSON data.
#[derive(Deserialize, Default)]
struct CreateSessionPayload {
    agent_type: Option<String>,
    session_summary_id: Option<String>,
    model: Option<String>,
    parent_session_id: Option<String>,
    prompt_summary: Option<String>,
    project_id: Option<String>,
    worktree_id: Option<String>,
    repo_path: Option<String>,
    operator_id: Option<String>,
    machine_id: Option<String>,
    agent_program: Option<String>,
    model_id: Option<String>,
}

/// Resolve the canonical origin context for a new agent session.
fn resolve_create_origin_context(
    args: &SessionArgs,
    payload: &CreateSessionPayload,
    session_id: &str,
    now: i64,
) -> Result<OriginContext, McpError> {
    resolve_origin_context(&OriginContextInput {
        org_id: args.org_id.as_deref(),
        project_id_args: args.project_id.as_deref(),
        project_id_payload: payload.project_id.as_deref(),
        session_from_args: Some(session_id),
        parent_session_from_args: args.parent_session_id.as_deref(),
        parent_session_from_data: payload.parent_session_id.as_deref(),
        tool_name_args: Some("session"),
        repo_path_payload: payload.repo_path.as_deref(),
        worktree_id_args: args.worktree_id.as_deref(),
        worktree_id_payload: payload.worktree_id.as_deref(),
        operator_id_payload: payload.operator_id.as_deref(),
        machine_id_payload: payload.machine_id.as_deref(),
        agent_program_payload: payload.agent_program.as_deref(),
        model_id_payload: payload.model_id.as_deref(),
        delegated_payload: Some(payload.parent_session_id.is_some()),
        require_project_id: true,
        timestamp: Some(now),
        ..Default::default()
    })
}

/// Parse the JSON `data` payload for session creation.
fn parse_create_payload(args: &SessionArgs) -> Result<CreateSessionPayload, McpError> {
    serde_json::from_value::<CreateSessionPayload>(
        args.data
            .clone()
            .ok_or_else(|| McpError::invalid_params("missing required field: data", None))?,
    )
    .map_err(|e| McpError::invalid_params(format!("invalid session data: {e}"), None))
}

/// Resolve the agent type from args/payload precedence, erroring if absent.
fn resolve_agent_type(
    args: &SessionArgs,
    payload: &CreateSessionPayload,
) -> Result<Result<AgentType, CallToolResult>, McpError> {
    let agent_type_value = resolve_identifier_precedence(
        "agent_type",
        args.agent_type.as_deref(),
        payload.agent_type.as_deref(),
    )?;
    match agent_type_value {
        Some(value) => Ok(Ok(parse_agent_type(&value)?)),
        None => Ok(Err(tool_error(
            "Missing agent_type for create (expected in args or data)",
        ))),
    }
}

/// Resolved inputs for assembling a new [`AgentSession`].
struct BuildSessionParams {
    session_id: String,
    agent_type: AgentType,
    model: String,
    now: i64,
    payload: CreateSessionPayload,
    origin_context: OriginContext,
}

/// Assemble an [`AgentSession`] in its initial active state.
fn build_agent_session(params: BuildSessionParams) -> AgentSession {
    let session_summary_id = params
        .payload
        .session_summary_id
        .unwrap_or_else(|| format!("auto_{}", domain_id::generate().simple()));
    AgentSession {
        id: params.session_id,
        session_summary_id,
        agent_type: params.agent_type,
        model: params.model,
        parent_session_id: params.payload.parent_session_id,
        started_at: params.now,
        ended_at: None,
        duration_ms: None,
        status: AgentSessionStatus::Active,
        prompt_summary: params.payload.prompt_summary,
        result_summary: None,
        token_count: None,
        tool_calls_count: None,
        delegations_count: None,
        project_id: params.origin_context.project_id,
        worktree_id: params.origin_context.worktree_id,
    }
}

/// Creates a new agent session.
#[tracing::instrument(skip_all)]
pub async fn create_session(
    agent_service: &Arc<dyn AgentSessionServiceInterface>,
    args: &SessionArgs,
) -> Result<CallToolResult, McpError> {
    let data = match require_data_map(&args.data, "Missing data payload for create") {
        Ok(data) => data,
        Err(error_result) => return Ok(error_result),
    };
    let payload = parse_create_payload(args)?;

    let agent_type: AgentType = match resolve_agent_type(args, &payload)? {
        Ok(value) => value,
        Err(error_result) => return Ok(error_result),
    };
    let now = mcb_utils::utils::time::epoch_secs_i64()
        .map_err(|e| safe_internal_error("resolve timestamp", &e))?;
    let session_id = domain_id::generate().to_string();
    let origin_context = resolve_create_origin_context(args, &payload, session_id.as_str(), now)?;
    let model = match payload.model.clone() {
        Some(value) => value,
        None => match require_str(data, schema::MODEL) {
            Ok(value) => value,
            Err(error_result) => return Ok(error_result),
        },
    };
    let session = build_agent_session(BuildSessionParams {
        session_id: session_id.clone(),
        agent_type: agent_type.clone(),
        model,
        now,
        payload,
        origin_context,
    });
    match agent_service.create_session(session).await {
        Ok(id) => ResponseFormatter::json_success(&serde_json::json!({
            "session_id": id,
            "agent_type": agent_type.as_str(),
            "status": "active",
        })),
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}
