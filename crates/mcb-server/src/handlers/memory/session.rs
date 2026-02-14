use std::sync::Arc;

use mcb_domain::ports::services::{CreateSessionSummaryInput, MemoryServiceInterface};
use mcb_domain::value_objects::SessionId;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use super::common::{require_data_map, str_vec};
use crate::args::MemoryArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handlers::helpers::{OriginContextInput, opt_bool, opt_str, resolve_origin_context};

/// Stores a session summary in the memory service.
#[tracing::instrument(skip_all)]
pub async fn store_session(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let data = match require_data_map(&args.data, "Missing data payload for session summary") {
        Ok(data) => data,
        Err(error_result) => return Ok(error_result),
    };
    let session_id = args
        .session_id
        .clone()
        .or_else(|| opt_str(data, "session_id").as_deref().map(SessionId::from));
    let session_id = match session_id {
        Some(value) => value,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing session_id for session summary",
            )]));
        }
    };
    let topics = str_vec(data, "topics");
    let decisions = str_vec(data, "decisions");
    let next_steps = str_vec(data, "next_steps");
    let key_files = str_vec(data, "key_files");
    let payload_project_id = opt_str(data, "project_id");
    let payload_parent_session_id = opt_str(data, "parent_session_id");
    let payload_repo_path = opt_str(data, "repo_path");
    let payload_worktree_id = opt_str(data, "worktree_id");
    let payload_operator_id = opt_str(data, "operator_id");
    let payload_machine_id = opt_str(data, "machine_id");
    let payload_agent_program = opt_str(data, "agent_program");
    let payload_model_id = opt_str(data, "model_id");
    let payload_delegated = opt_bool(data, "delegated");
    let session_id_str = session_id.as_str().to_owned();

    let origin_context = resolve_origin_context(OriginContextInput {
        org_id: args.org_id.as_deref(),
        project_id_args: args.project_id.as_deref(),
        project_id_payload: payload_project_id.as_deref(),
        session_from_args: Some(session_id_str.as_str()),
        session_from_data: None,
        parent_session_from_args: None,
        parent_session_from_data: payload_parent_session_id.as_deref(),
        execution_from_args: None,
        execution_from_data: None,
        tool_name_args: Some("memory"),
        tool_name_payload: None,
        repo_id_args: args.repo_id.as_deref(),
        repo_id_payload: None,
        repo_path_args: None,
        repo_path_payload: payload_repo_path.as_deref(),
        worktree_id_args: None,
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
        delegated_payload: payload_delegated,
        require_project_id: true,
        timestamp: None,
    })?;
    let project_id = origin_context.project_id.clone().ok_or_else(|| {
        McpError::invalid_params("project_id is required for session summary", None)
    })?;
    match memory_service
        .create_session_summary(CreateSessionSummaryInput {
            project_id,
            session_id,
            topics,
            decisions,
            next_steps,
            key_files,
            origin_context: Some(origin_context),
        })
        .await
    {
        Ok(summary_id) => ResponseFormatter::json_success(&serde_json::json!({
            "summary_id": summary_id,
            "session_id": session_id_str,
        })),
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}

/// Retrieves a session summary from the memory service.
#[tracing::instrument(skip_all)]
pub async fn get_session(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let session_id = match args.session_id.as_ref() {
        Some(value) => value,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing session_id",
            )]));
        }
    };
    match memory_service.get_session_summary(session_id).await {
        Ok(Some(summary)) => ResponseFormatter::json_success(&serde_json::json!({
            "session_id": summary.session_id,
            "topics": summary.topics,
            "decisions": summary.decisions,
            "next_steps": summary.next_steps,
            "key_files": summary.key_files,
            "created_at": summary.created_at,
        })),
        Ok(None) => Ok(CallToolResult::error(vec![Content::text(
            "Session summary not found",
        )])),
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}
