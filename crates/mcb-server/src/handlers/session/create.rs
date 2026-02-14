use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mcb_domain::constants::keys as schema;

use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus, AgentType};
use mcb_domain::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::Value;
use uuid::Uuid;

use crate::args::SessionArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handler_helpers::{
    OriginContextInput, resolve_identifier_precedence, resolve_origin_context,
};
use crate::utils::json;
use tracing::error;

/// Creates a new agent session.
#[tracing::instrument(skip_all)]
pub async fn create_session(
    agent_service: &Arc<dyn AgentSessionServiceInterface>,
    args: &SessionArgs,
) -> Result<CallToolResult, McpError> {
    let data = match json::json_map(&args.data) {
        Some(data) => data,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing data payload for create",
            )]));
        }
    };

    let payload_agent_type = data
        .get("agent_type")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let agent_type_value = resolve_identifier_precedence(
        "agent_type",
        args.agent_type.as_deref(),
        payload_agent_type.as_deref(),
    )?;
    let agent_type: AgentType = match agent_type_value {
        Some(value) => value
            .parse::<AgentType>()
            .map_err(|e: String| McpError::invalid_params(e, None))?,
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
    let session_summary_id = data
        .get(schema::SESSION_SUMMARY_ID)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("auto_{}", Uuid::new_v4()));
    let model = match data.get(schema::MODEL).and_then(Value::as_str) {
        Some(value) => value.to_owned(),
        None => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Missing required field: {}",
                schema::MODEL
            ))]));
        }
    };
    let session = AgentSession {
        id: session_id.clone(),
        session_summary_id,
        agent_type: agent_type.clone(),
        model,
        parent_session_id: data
            .get(schema::PARENT_SESSION_ID)
            .and_then(Value::as_str)
            .map(str::to_owned),
        started_at: now,
        ended_at: None,
        duration_ms: None,
        status: AgentSessionStatus::Active,
        prompt_summary: data
            .get(schema::PROMPT_SUMMARY)
            .and_then(Value::as_str)
            .map(str::to_owned),
        result_summary: None,
        token_count: None,
        tool_calls_count: None,
        delegations_count: None,
        project_id: None,
        worktree_id: None,
    };
    let payload_project_id = data
        .get(schema::PROJECT_ID)
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_worktree_id = data
        .get(schema::WORKTREE_ID)
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_parent_session_id = data
        .get(schema::PARENT_SESSION_ID)
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_repo_path = data
        .get(schema::REPO_PATH)
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_operator_id = data
        .get("operator_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_machine_id = data
        .get("machine_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_agent_program = data
        .get("agent_program")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let payload_model_id = data
        .get("model_id")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let origin_context = resolve_origin_context(OriginContextInput {
        org_id: args.org_id.as_deref(),
        project_id_args: args.project_id.as_deref(),
        project_id_payload: payload_project_id.as_deref(),
        session_from_args: Some(session_id.as_str()),
        session_from_data: None,
        parent_session_from_args: args.parent_session_id.as_deref(),
        parent_session_from_data: payload_parent_session_id.as_deref(),
        execution_from_args: None,
        execution_from_data: None,
        tool_name_args: Some("session"),
        tool_name_payload: None,
        repo_id_args: None,
        repo_id_payload: None,
        repo_path_args: None,
        repo_path_payload: payload_repo_path.as_deref(),
        worktree_id_args: args.worktree_id.as_deref(),
        worktree_id_payload: payload_worktree_id.as_deref(),
        file_path_args: None,
        file_path_payload: None,
        branch_args: None,
        branch_payload: None,
        commit_args: None,
        commit_payload: None,
        operator_id_args: None,
        operator_id_payload: payload_operator_id.as_deref(),
        machine_id_args: None,
        machine_id_payload: payload_machine_id.as_deref(),
        agent_program_args: None,
        agent_program_payload: payload_agent_program.as_deref(),
        model_id_args: None,
        model_id_payload: payload_model_id.as_deref(),
        delegated_args: None,
        delegated_payload: Some(payload_parent_session_id.is_some()),
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
