use crate::args::StoreToolCallArgs;
use crate::formatter::ResponseFormatter;
use mcb_application::ports::services::AgentSessionServiceInterface;
use mcb_domain::entities::agent::ToolCall;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use validator::Validate;

/// MCP handler for storing a tool call in an agent session.
pub struct StoreToolCallHandler {
    service: Arc<dyn AgentSessionServiceInterface>,
}

#[derive(Serialize)]
struct StoreResult {
    tool_call_id: String,
    session_id: String,
    tool_name: String,
}

impl StoreToolCallHandler {
    pub fn new(service: Arc<dyn AgentSessionServiceInterface>) -> Self {
        Self { service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<StoreToolCallArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let tool_call_id = format!("tc_{}", uuid::Uuid::new_v4());

        let tool_call = ToolCall {
            id: tool_call_id.clone(),
            session_id: args.session_id.clone(),
            tool_name: args.tool_name.clone(),
            params_summary: args.params_summary,
            success: args.success,
            error_message: args.error_message,
            duration_ms: args.duration_ms,
            created_at: now,
        };

        match self.service.store_tool_call(tool_call).await {
            Ok(_) => {
                let result = StoreResult {
                    tool_call_id,
                    session_id: args.session_id,
                    tool_name: args.tool_name,
                };
                ResponseFormatter::json_success(&result)
            }
            Err(_) => Ok(CallToolResult::error(vec![Content::text(
                "Failed to store tool call",
            )])),
        }
    }
}
