use crate::args::SessionArgs;
use crate::formatter::ResponseFormatter;
use mcb_application::ports::services::AgentSessionServiceInterface;
use mcb_application::services::AgentSessionQuery;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use std::sync::Arc;

pub async fn list_sessions(
    agent_service: &Arc<dyn AgentSessionServiceInterface>,
    args: &SessionArgs,
) -> Result<CallToolResult, McpError> {
    let query = AgentSessionQuery {
        session_summary_id: None,
        parent_session_id: None,
        agent_type: args
            .agent_type
            .as_ref()
            .map(|value| value.parse())
            .transpose()
            .map_err(|_| McpError::invalid_params("Invalid agent_type", None))?,
        status: args
            .status
            .as_ref()
            .map(|value| value.parse())
            .transpose()
            .map_err(|_| McpError::invalid_params("Invalid status", None))?,
        limit: Some(args.limit.unwrap_or(10) as usize),
    };
    match agent_service.list_sessions(query).await {
        Ok(sessions) => {
            let items: Vec<_> = sessions
                .iter()
                .map(|session| {
                    serde_json::json!({
                        "id": session.id,
                        "agent_type": session.agent_type.as_str(),
                        "status": session.status.as_str(),
                        "started_at": session.started_at,
                        "duration_ms": session.duration_ms,
                    })
                })
                .collect();
            ResponseFormatter::json_success(&serde_json::json!({
                "sessions": items,
                "count": items.len(),
            }))
        }
        Err(_) => Ok(CallToolResult::error(vec![Content::text(
            "Failed to list agent sessions",
        )])),
    }
}
