//! Agent handler for tool call and delegation logging.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mcb_domain::entities::agent::{Delegation, ToolCall};
use mcb_domain::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};

use uuid::Uuid;
use validator::Validate;

use crate::args::{AgentAction, AgentArgs};
use crate::formatter::ResponseFormatter;
use crate::utils::json::{get_bool, get_i64, get_str};

/// Handler for agent tool call and delegation logging operations.
#[derive(Clone)]
pub struct AgentHandler {
    agent_service: Arc<dyn AgentSessionServiceInterface>,
}

impl AgentHandler {
    /// Create a new AgentHandler.
    pub fn new(agent_service: Arc<dyn AgentSessionServiceInterface>) -> Self {
        Self { agent_service }
    }

    /// Handle an agent tool request.
    pub async fn handle(
        &self,
        Parameters(args): Parameters<AgentArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;

        if args.session_id.to_string().is_empty() {
            return Err(McpError::invalid_params("session_id is required", None));
        }

        let data = match args.data.as_object() {
            Some(data) => data,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(
                    "Data must be a JSON object",
                )]));
            }
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        match args.action {
            AgentAction::LogTool => {
                let tool_name = match get_str(data, "tool_name") {
                    Some(value) => value,
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing tool_name",
                        )]));
                    }
                };
                let tool_call = ToolCall {
                    id: format!("tc_{}", Uuid::new_v4()),
                    session_id: args.session_id.to_string(),
                    tool_name: tool_name.clone(),
                    params_summary: get_str(data, "params_summary"),
                    success: get_bool(data, "success").unwrap_or(true),
                    error_message: get_str(data, "error_message"),
                    duration_ms: get_i64(data, "duration_ms"),
                    created_at: now,
                };
                match self.agent_service.store_tool_call(tool_call).await {
                    Ok(id) => ResponseFormatter::json_success(&serde_json::json!({
                        "tool_call_id": id,
                        "session_id": args.session_id.to_string(),
                        "tool_name": tool_name,
                    })),
                    Err(_) => Ok(CallToolResult::error(vec![Content::text(
                        "Failed to store tool call",
                    )])),
                }
            }
            AgentAction::LogDelegation => {
                let child_session_id = match get_str(data, "child_session_id") {
                    Some(value) => value,
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing child_session_id",
                        )]));
                    }
                };
                let delegation = Delegation {
                    id: format!("del_{}", Uuid::new_v4()),
                    parent_session_id: args.session_id.to_string(),
                    child_session_id: child_session_id.clone(),
                    prompt: get_str(data, "prompt").unwrap_or_default(),
                    prompt_embedding_id: get_str(data, "prompt_embedding_id"),
                    result: get_str(data, "result"),
                    success: get_bool(data, "success").unwrap_or(true),
                    created_at: now,
                    completed_at: None,
                    duration_ms: get_i64(data, "duration_ms"),
                };
                match self.agent_service.store_delegation(delegation).await {
                    Ok(id) => ResponseFormatter::json_success(&serde_json::json!({
                        "delegation_id": id,
                        "parent_session_id": args.session_id.to_string(),
                        "child_session_id": child_session_id,
                    })),
                    Err(_) => Ok(CallToolResult::error(vec![Content::text(
                        "Failed to store delegation",
                    )])),
                }
            }
        }
    }
}
