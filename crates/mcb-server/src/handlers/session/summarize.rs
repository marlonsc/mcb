use std::sync::Arc;

use mcb_domain::ports::services::MemoryServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use super::helpers::SessionHelpers;
use crate::args::SessionArgs;
use crate::error_mapping::to_opaque_tool_error;
use crate::formatter::ResponseFormatter;

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
        match memory_service
            .create_session_summary(session_id.clone(), topics, decisions, next_steps, key_files)
            .await
        {
            Ok(summary_id) => ResponseFormatter::json_success(&serde_json::json!({
                "summary_id": summary_id,
                "session_id": session_id.as_str(),
            })),
            Err(e) => Ok(to_opaque_tool_error(e)),
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
            Err(e) => Ok(to_opaque_tool_error(e)),
        }
    }
}
