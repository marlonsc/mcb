use crate::args::CreateAgentSessionArgs;
use crate::formatter::ResponseFormatter;
use mcb_application::ports::services::AgentSessionServiceInterface;
use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus, AgentType};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use validator::Validate;

/// MCP handler for creating a new agent session.
pub struct CreateAgentSessionHandler {
    service: Arc<dyn AgentSessionServiceInterface>,
}

#[derive(Serialize)]
struct CreateSessionResult {
    session_id: String,
    agent_type: String,
    status: String,
}

impl CreateAgentSessionHandler {
    pub fn new(service: Arc<dyn AgentSessionServiceInterface>) -> Self {
        Self { service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<CreateAgentSessionArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let agent_type: AgentType = args
            .agent_type
            .parse()
            .map_err(|_| McpError::invalid_params("Invalid agent_type", None))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let session_id = format!("agent_{}", uuid::Uuid::new_v4());

        let session = AgentSession {
            id: session_id.clone(),
            session_summary_id: args.session_summary_id,
            agent_type: agent_type.clone(),
            model: args.model,
            parent_session_id: args.parent_session_id,
            started_at: now,
            ended_at: None,
            duration_ms: None,
            status: AgentSessionStatus::Active,
            prompt_summary: args.prompt_summary,
            result_summary: None,
            token_count: None,
            tool_calls_count: None,
            delegations_count: None,
        };

        match self.service.create_session(session).await {
            Ok(id) => {
                let result = CreateSessionResult {
                    session_id: id,
                    agent_type: agent_type.as_str().to_string(),
                    status: "active".to_string(),
                };
                ResponseFormatter::json_success(&result)
            }
            Err(_) => Ok(CallToolResult::error(vec![Content::text(
                "Failed to create agent session",
            )])),
        }
    }
}
