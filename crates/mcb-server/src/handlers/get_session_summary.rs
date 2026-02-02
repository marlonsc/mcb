//! Handler for the `get_session_summary` MCP tool

use crate::args::GetSessionSummaryArgs;
use mcb_application::ports::MemoryServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

pub struct GetSessionSummaryHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

#[derive(Serialize)]
struct SessionSummaryResponse {
    session_id: String,
    topics: Vec<String>,
    decisions: Vec<String>,
    next_steps: Vec<String>,
    key_files: Vec<String>,
    created_at: i64,
}

impl GetSessionSummaryHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<GetSessionSummaryArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        match self
            .memory_service
            .get_session_summary(&args.session_id)
            .await
        {
            Ok(Some(summary)) => {
                let response = SessionSummaryResponse {
                    session_id: summary.session_id,
                    topics: summary.topics,
                    decisions: summary.decisions,
                    next_steps: summary.next_steps,
                    key_files: summary.key_files,
                    created_at: summary.created_at,
                };

                let json = serde_json::to_string_pretty(&response)
                    .unwrap_or_else(|_| "Failed to serialize summary".to_string());

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Ok(None) => Ok(CallToolResult::error(vec![Content::text(format!(
                "No session summary found for session: {}",
                args.session_id
            ))])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to get session summary: {e}"
            ))])),
        }
    }
}
