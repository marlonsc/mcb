//! Handler for the `create_session_summary` MCP tool

use crate::args::CreateSessionSummaryArgs;
use mcb_application::ports::MemoryServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

/// Handler for the MCP `create_session_summary` tool (semantic memory).
pub struct CreateSessionSummaryHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

#[derive(Serialize)]
struct CreateSummaryResult {
    summary_id: String,
    session_id: String,
}

impl CreateSessionSummaryHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<CreateSessionSummaryArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        match self
            .memory_service
            .create_session_summary(
                args.session_id.clone(),
                args.topics,
                args.decisions,
                args.next_steps,
                args.key_files,
            )
            .await
        {
            Ok(id) => {
                let result = CreateSummaryResult {
                    summary_id: id,
                    session_id: args.session_id,
                };

                let json = serde_json::to_string_pretty(&result)
                    .unwrap_or_else(|_| String::from("Failed to serialize result"));

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(_) => Ok(CallToolResult::error(vec![Content::text(
                "Failed to create session summary",
            )])),
        }
    }
}
