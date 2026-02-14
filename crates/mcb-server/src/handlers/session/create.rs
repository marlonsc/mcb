use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mcb_domain::constants::keys as schema;

use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus, AgentType};
use mcb_domain::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::Deserialize;
use uuid::Uuid;

use super::common::{parse_agent_type, require_data_map, require_str};
use crate::args::SessionArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handlers::helpers::{
    OriginContextInput, resolve_identifier_precedence, resolve_origin_context,
};
use tracing::error;

/// Payload for creating an agent session from JSON data.
#[derive(Deserialize, Default)]
#[serde(default)]
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
    let payload =
        serde_json::from_value::<CreateSessionPayload>(args.data.clone().unwrap_or_default())
            .map_err(|_| McpError::invalid_params("invalid data", None))?;

    let agent_type_value = resolve_identifier_precedence(
        "agent_type",
        args.agent_type.as_deref(),
        payload.agent_type.as_deref(),
    )?;
    let agent_type: AgentType = match agent_type_value {
        Some(value) => parse_agent_type(&value)?,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing agent_type for create (expected in args or data)",
            )]));
        }
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let session_id = format!("agent_{}", Uuid::new_v4());
    let session_summary_id = payload
        .session_summary_id
        .unwrap_or_else(|| format!("auto_{}", Uuid::new_v4()));
    let model = match payload.model {
        Some(value) => value,
        None => match require_str(data, schema::MODEL) {
            Ok(value) => value,
            Err(error_result) => return Ok(error_result),
        },
    };
    let session = AgentSession {
        id: session_id.clone(),
        session_summary_id,
        agent_type: agent_type.clone(),
        model,
        parent_session_id: payload.parent_session_id.clone(),
        started_at: now,
        ended_at: None,
        duration_ms: None,
        status: AgentSessionStatus::Active,
        prompt_summary: payload.prompt_summary,
        result_summary: None,
        token_count: None,
        tool_calls_count: None,
        delegations_count: None,
        project_id: None,
        worktree_id: None,
    };
    let origin_context = resolve_origin_context(OriginContextInput {
        org_id: args.org_id.as_deref(),
        project_id_args: args.project_id.as_deref(),
        project_id_payload: payload.project_id.as_deref(),
        session_from_args: Some(session_id.as_str()),
        session_from_data: None,
        parent_session_from_args: args.parent_session_id.as_deref(),
        parent_session_from_data: payload.parent_session_id.as_deref(),
        execution_from_args: None,
        execution_from_data: None,
        tool_name_args: Some("session"),
        tool_name_payload: None,
        repo_id_args: None,
        repo_id_payload: None,
        repo_path_args: None,
        repo_path_payload: payload.repo_path.as_deref(),
        worktree_id_args: args.worktree_id.as_deref(),
        worktree_id_payload: payload.worktree_id.as_deref(),
        file_path_args: None,
        file_path_payload: None,
        branch_args: None,
        branch_payload: None,
        commit_args: None,
        commit_payload: None,
        operator_id_args: None,
        operator_id_payload: payload.operator_id.as_deref(),
        machine_id_args: None,
        machine_id_payload: payload.machine_id.as_deref(),
        agent_program_args: None,
        agent_program_payload: payload.agent_program.as_deref(),
        model_id_args: None,
        model_id_payload: payload.model_id.as_deref(),
        delegated_args: None,
        delegated_payload: Some(payload.parent_session_id.is_some()),
        require_project_id: true,
        timestamp: Some(now),
    })?;
    let session = AgentSession {
        project_id: origin_context.project_id,
        worktree_id: origin_context.worktree_id,
        ..session
    };
    match agent_service.create_session(session).await {
        Ok(id) => ResponseFormatter::json_success(&serde_json::json!({
            "session_id": id,
            "agent_type": agent_type.as_str(),
            "status": "active",
        })),
        Err(e) => {
            error!("Failed to create agent session: {:?}", e);
            Ok(to_contextual_tool_error(e))
        }
    }
}
