use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mcb_domain::constants::keys as schema;
use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus};
use mcb_domain::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use uuid::Uuid;

use super::helpers::SessionHelpers;
use crate::args::SessionArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handler_helpers::{
    OriginContextInput, resolve_identifier_precedence, resolve_origin_context,
};
use tracing::error;

/// Creates a new agent session.
#[tracing::instrument(skip_all)]
pub async fn create_session(
    agent_service: &Arc<dyn AgentSessionServiceInterface>,
    args: &SessionArgs,
) -> Result<CallToolResult, McpError> {
    let data = match SessionHelpers::json_map(&args.data) {
        Some(data) => data,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing data payload for create",
            )]));
        }
    };

    let payload_agent_type = SessionHelpers::get_str(data, "agent_type");
    let agent_type_value = resolve_identifier_precedence(
        "agent_type",
        args.agent_type.as_deref(),
        payload_agent_type.as_deref(),
    )?;
    let agent_type = match agent_type_value {
        Some(value) => SessionHelpers::parse_agent_type(&value)?,
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
    let session_summary_id = SessionHelpers::get_str(data, schema::SESSION_SUMMARY_ID)
        .unwrap_or_else(|| format!("auto_{}", Uuid::new_v4()));
    let model = match SessionHelpers::get_required_str(data, schema::MODEL) {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let session = AgentSession {
        id: session_id.clone(),
        session_summary_id,
        agent_type: agent_type.clone(),
        model,
        parent_session_id: SessionHelpers::get_str(data, schema::PARENT_SESSION_ID),
        started_at: now,
        ended_at: None,
        duration_ms: None,
        status: AgentSessionStatus::Active,
        prompt_summary: SessionHelpers::get_str(data, schema::PROMPT_SUMMARY),
        result_summary: None,
        token_count: None,
        tool_calls_count: None,
        delegations_count: None,
        project_id: None,
        worktree_id: None,
    };
    let payload_project_id = SessionHelpers::get_str(data, schema::PROJECT_ID);
    let payload_worktree_id = SessionHelpers::get_str(data, schema::WORKTREE_ID);
    let origin_context = resolve_origin_context(OriginContextInput {
        org_id: args.org_id.as_deref(),
        project_id_args: args.project_id.as_deref(),
        project_id_payload: payload_project_id.as_deref(),
        session_id_args: Some(session_id.as_str()),
        session_id_payload: None,
        execution_id_args: None,
        execution_id_payload: None,
        tool_name_args: Some("session"),
        tool_name_payload: None,
        repo_id_args: None,
        repo_id_payload: None,
        repo_path_args: None,
        repo_path_payload: None,
        worktree_id_args: args.worktree_id.as_deref(),
        worktree_id_payload: payload_worktree_id.as_deref(),
        file_path_args: None,
        file_path_payload: None,
        branch_args: None,
        branch_payload: None,
        commit_args: None,
        commit_payload: None,
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
