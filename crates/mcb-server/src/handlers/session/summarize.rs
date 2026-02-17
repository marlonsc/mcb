use std::sync::Arc;

use mcb_domain::ports::{CreateSessionSummaryInput, MemoryServiceInterface};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::common::{json_map, str_vec};
use crate::args::SessionArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::utils::mcp::{OriginPayloadFields, resolve_origin_context, tool_error};

/// Creates or retrieves a session summary.
#[tracing::instrument(skip_all)]
pub async fn summarize_session(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &SessionArgs,
) -> Result<CallToolResult, McpError> {
    let session_id = match args.session_id.as_ref() {
        Some(id) => id,
        None => {
            return Ok(tool_error("Missing session_id"));
        }
    };
    if let Some(data) = json_map(&args.data) {
        let session_id_str = session_id.to_string();
        let topics = str_vec(data, "topics");
        let decisions = str_vec(data, "decisions");
        let next_steps = str_vec(data, "next_steps");
        let key_files = str_vec(data, "key_files");
        let payload = OriginPayloadFields::extract(data);
        let mut input = payload.to_input();
        input.org_id = args.org_id.as_deref();
        input.project_id_args = args.project_id.as_deref();
        input.session_from_args = Some(&session_id_str);
        input.parent_session_from_args = args.parent_session_id.as_deref();
        input.tool_name_args = Some("session");
        input.worktree_id_args = args.worktree_id.as_deref();
        input.require_project_id = true;
        let origin_context = resolve_origin_context(&input)?;
        let project_id = origin_context.project_id.clone().ok_or_else(|| {
            McpError::invalid_params("project_id is required for summarize", None)
        })?;
        match memory_service
            .create_session_summary(CreateSessionSummaryInput {
                project_id,
                session_id: *session_id,
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
            Ok(None) => Ok(tool_error("Session summary not found")),
            Err(e) => Ok(to_contextual_tool_error(e)),
        }
    }
}
