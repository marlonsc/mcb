use std::sync::Arc;

use mcb_domain::ports::services::{CreateSessionSummaryInput, MemoryServiceInterface};
use mcb_domain::value_objects::SessionId;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::Deserialize;

use crate::args::MemoryArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handlers::helpers::{OriginContextInput, resolve_origin_context};

/// Payload for storing a session summary in memory.
#[derive(Deserialize, Default)]
#[serde(default)]
struct SessionSummaryPayload {
    session_id: Option<String>,
    #[serde(default)]
    topics: Vec<String>,
    #[serde(default)]
    decisions: Vec<String>,
    #[serde(default)]
    next_steps: Vec<String>,
    #[serde(default)]
    key_files: Vec<String>,
    project_id: Option<String>,
    parent_session_id: Option<String>,
    repo_path: Option<String>,
    worktree_id: Option<String>,
    operator_id: Option<String>,
    machine_id: Option<String>,
    agent_program: Option<String>,
    model_id: Option<String>,
    delegated: Option<bool>,
}

/// Stores a session summary in the memory service.
#[tracing::instrument(skip_all)]
pub async fn store_session(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    if args.data.is_none() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Missing data payload for session summary",
        )]));
    }
    let payload =
        serde_json::from_value::<SessionSummaryPayload>(args.data.clone().unwrap_or_default())
            .map_err(|_| McpError::invalid_params("invalid data", None))?;
    let session_id = args
        .session_id
        .clone()
        .or_else(|| payload.session_id.as_deref().map(SessionId::from));
    let session_id = match session_id {
        Some(value) => value,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing session_id for session summary",
            )]));
        }
    };
    let session_id_str = session_id.as_str().to_owned();

    let origin_context = resolve_origin_context(OriginContextInput {
        org_id: args.org_id.as_deref(),
        project_id_args: args.project_id.as_deref(),
        project_id_payload: payload.project_id.as_deref(),
        session_from_args: Some(session_id_str.as_str()),
        session_from_data: None,
        parent_session_from_args: None,
        parent_session_from_data: payload.parent_session_id.as_deref(),
        execution_from_args: None,
        execution_from_data: None,
        tool_name_args: Some("memory"),
        tool_name_payload: None,
        repo_id_args: args.repo_id.as_deref(),
        repo_id_payload: None,
        repo_path_args: None,
        repo_path_payload: payload.repo_path.as_deref(),
        worktree_id_args: None,
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
        delegated_payload: payload.delegated,
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
            topics: payload.topics,
            decisions: payload.decisions,
            next_steps: payload.next_steps,
            key_files: payload.key_files,
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
