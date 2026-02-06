use crate::args::SessionArgs;
use crate::formatter::ResponseFormatter;
use mcb_application::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use std::sync::Arc;

use mcb_domain::constants::keys as schema;

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
            schema::ID: session.id,
            schema::SESSION_SUMMARY_ID: session.session_summary_id,
            schema::AGENT_TYPE: session.agent_type.as_str(),
            schema::MODEL: session.model,
            schema::PARENT_SESSION_ID: session.parent_session_id,
            schema::STARTED_AT: session.started_at,
            schema::ENDED_AT: session.ended_at,
            schema::DURATION_MS: session.duration_ms,
            schema::STATUS: session.status.as_str(),
            schema::PROMPT_SUMMARY: session.prompt_summary,
            schema::RESULT_SUMMARY: session.result_summary,
            schema::TOKEN_COUNT: session.token_count,
            schema::TOOL_CALLS_COUNT: session.tool_calls_count,
            schema::DELEGATIONS_COUNT: session.delegations_count,
        })),
        Ok(None) => Ok(CallToolResult::error(vec![Content::text(
            "Agent session not found",
        )])),
        Err(_) => Ok(CallToolResult::error(vec![Content::text(
            "Failed to get agent session",
        )])),
    }
}
