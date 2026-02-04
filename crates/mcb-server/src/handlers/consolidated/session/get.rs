use crate::args::SessionArgs;
use crate::formatter::ResponseFormatter;
use mcb_application::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use std::sync::Arc;

pub async fn get_session(
    agent_service: &Arc<dyn AgentSessionServiceInterface>,
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
    match agent_service.get_session(session_id).await {
        Ok(Some(session)) => ResponseFormatter::json_success(&serde_json::json!({
            "id": session.id,
            "session_summary_id": session.session_summary_id,
            "agent_type": session.agent_type.as_str(),
            "model": session.model,
            "parent_session_id": session.parent_session_id,
            "started_at": session.started_at,
            "ended_at": session.ended_at,
            "duration_ms": session.duration_ms,
            "status": session.status.as_str(),
            "prompt_summary": session.prompt_summary,
            "result_summary": session.result_summary,
            "token_count": session.token_count,
            "tool_calls_count": session.tool_calls_count,
            "delegations_count": session.delegations_count,
        })),
        Ok(None) => Ok(CallToolResult::error(vec![Content::text(
            "Agent session not found",
        )])),
        Err(_) => Ok(CallToolResult::error(vec![Content::text(
            "Failed to get agent session",
        )])),
    }
}
