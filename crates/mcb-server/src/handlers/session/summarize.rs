use std::sync::Arc;

use mcb_domain::ports::services::{CreateSessionSummaryInput, MemoryServiceInterface};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use super::helpers::SessionHelpers;
use crate::args::SessionArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handler_helpers::{OriginContextInput, resolve_origin_context};

/// Creates or retrieves a session summary.
#[tracing::instrument(skip_all)]
pub async fn summarize_session(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &SessionArgs,
) -> Result<CallToolResult, McpError> {
    let session_id = match args.session_id.as_ref() {
        Some(id) => id,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing session_id",
            )]));
        }
    };
    if let Some(data) = SessionHelpers::json_map(&args.data) {
        let topics = SessionHelpers::get_string_list(data, "topics");
        let decisions = SessionHelpers::get_string_list(data, "decisions");
        let next_steps = SessionHelpers::get_string_list(data, "next_steps");
        let key_files = SessionHelpers::get_string_list(data, "key_files");
        let payload_project_id = SessionHelpers::get_str(data, "project_id");
        let payload_session_id = SessionHelpers::get_str(data, "session_id");
        let payload_worktree_id = SessionHelpers::get_str(data, "worktree_id");
        let origin_context = resolve_origin_context(OriginContextInput {
            org_id: args.org_id.as_deref(),
            project_id_args: args.project_id.as_deref(),
            project_id_payload: payload_project_id.as_deref(),
            session_id_args: Some(session_id.as_str()),
            session_id_payload: payload_session_id.as_deref(),
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
            timestamp: None,
        })?;
        let project_id = origin_context.project_id.clone().ok_or_else(|| {
            McpError::invalid_params("project_id is required for summarize", None)
        })?;
        match memory_service
            .create_session_summary(CreateSessionSummaryInput {
                project_id,
                session_id: session_id.clone(),
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
                "session_id": session_id.as_str(),
            })),
            Err(e) => Ok(to_contextual_tool_error(e)),
        }
    } else {
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
}
