use std::sync::Arc;

use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::value_objects::SessionId;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use super::helpers::MemoryHelpers;
use crate::args::MemoryArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;

/// Stores a session summary in the memory service.
#[tracing::instrument(skip_all)]
pub async fn store_session(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let data = match MemoryHelpers::json_map(&args.data) {
        Some(data) => data,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing data payload for session summary",
            )]));
        }
    };
    let session_id = args
        .session_id
        .clone()
        .or_else(|| MemoryHelpers::get_str(data, "session_id").map(SessionId::new));
    let session_id = match session_id {
        Some(value) => value,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing session_id for session summary",
            )]));
        }
    };
    let topics = MemoryHelpers::get_string_list(data, "topics");
    let decisions = MemoryHelpers::get_string_list(data, "decisions");
    let next_steps = MemoryHelpers::get_string_list(data, "next_steps");
    let key_files = MemoryHelpers::get_string_list(data, "key_files");
    let session_id_str = session_id.as_str().to_string();
    match memory_service
        .create_session_summary(session_id, topics, decisions, next_steps, key_files)
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
