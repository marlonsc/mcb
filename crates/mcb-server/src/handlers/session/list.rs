use std::sync::Arc;

use mcb_domain::ports::repositories::agent_repository::AgentSessionQuery;
use mcb_domain::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::args::SessionArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;

/// Lists agent sessions based on filters.
#[tracing::instrument(skip_all)]
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
            .and_then(|value| value.parse().ok()),
        status: args
            .status
            .as_ref()
            .filter(|value| !value.is_empty())
            .map(|value| value.parse())
            .transpose()
            .map_err(|_| McpError::invalid_params("Invalid status", None))?,
        project_id: args.project_id.clone(),
        worktree_id: args.worktree_id.clone(),
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
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}
