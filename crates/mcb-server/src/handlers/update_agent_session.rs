use crate::args::UpdateAgentSessionArgs;
use crate::formatter::ResponseFormatter;
use mcb_application::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

/// MCP handler for updating an existing agent session.
pub struct UpdateAgentSessionHandler {
    service: Arc<dyn AgentSessionServiceInterface>,
}

#[derive(Serialize)]
struct UpdateResult {
    id: String,
    status: String,
    updated: bool,
}

impl UpdateAgentSessionHandler {
    pub fn new(service: Arc<dyn AgentSessionServiceInterface>) -> Self {
        Self { service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<UpdateAgentSessionArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let session = self.service.get_session(&args.id).await;

        match session {
            Ok(Some(mut session)) => {
                if let Some(status_str) = &args.status {
                    session.status = status_str
                        .parse()
                        .map_err(|_| McpError::invalid_params("Invalid status", None))?;
                }

                if let Some(result_summary) = args.result_summary {
                    session.result_summary = Some(result_summary);
                }

                if let Some(token_count) = args.token_count {
                    session.token_count = Some(token_count);
                }

                if let Some(tool_calls_count) = args.tool_calls_count {
                    session.tool_calls_count = Some(tool_calls_count);
                }

                if let Some(delegations_count) = args.delegations_count {
                    session.delegations_count = Some(delegations_count);
                }

                let final_status = session.status.as_str().to_string();

                match self.service.update_session(session).await {
                    Ok(()) => {
                        let result = UpdateResult {
                            id: args.id,
                            status: final_status,
                            updated: true,
                        };
                        ResponseFormatter::json_success(&result)
                    }
                    Err(_) => Ok(CallToolResult::error(vec![Content::text(
                        "Failed to update agent session",
                    )])),
                }
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
