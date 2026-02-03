use crate::args::GetAgentSessionArgs;
use mcb_application::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

pub struct GetAgentSessionHandler {
    service: Arc<dyn AgentSessionServiceInterface>,
}

#[derive(Serialize)]
struct AgentSessionResult {
    id: String,
    session_summary_id: String,
    agent_type: String,
    model: String,
    parent_session_id: Option<String>,
    started_at: i64,
    ended_at: Option<i64>,
    duration_ms: Option<i64>,
    status: String,
    prompt_summary: Option<String>,
    result_summary: Option<String>,
    token_count: Option<i64>,
    tool_calls_count: Option<i64>,
    delegations_count: Option<i64>,
}

impl GetAgentSessionHandler {
    pub fn new(service: Arc<dyn AgentSessionServiceInterface>) -> Self {
        Self { service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<GetAgentSessionArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        match self.service.get_session(&args.id).await {
            Ok(Some(session)) => {
                let result = AgentSessionResult {
                    id: session.id,
                    session_summary_id: session.session_summary_id,
                    agent_type: session.agent_type.as_str().to_string(),
                    model: session.model,
                    parent_session_id: session.parent_session_id,
                    started_at: session.started_at,
                    ended_at: session.ended_at,
                    duration_ms: session.duration_ms,
                    status: session.status.as_str().to_string(),
                    prompt_summary: session.prompt_summary,
                    result_summary: session.result_summary,
                    token_count: session.token_count,
                    tool_calls_count: session.tool_calls_count,
                    delegations_count: session.delegations_count,
                };

                let json = serde_json::to_string_pretty(&result)
                    .unwrap_or_else(|_| String::from("Failed to serialize result"));

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Ok(None) => Ok(CallToolResult::error(vec![Content::text(
                "Agent session not found",
            )])),
            Err(_) => Ok(CallToolResult::error(vec![Content::text(
                "Failed to get agent session",
            )])),
        }
    }
}
