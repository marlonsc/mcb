use crate::args::{AgentAction, AgentArgs};
use crate::formatter::ResponseFormatter;
use mcb_application::ports::services::AgentSessionServiceInterface;
use mcb_domain::entities::agent::{Delegation, ToolCall};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use validator::Validate;

#[derive(Clone)]
pub struct AgentHandler {
    agent_service: Arc<dyn AgentSessionServiceInterface>,
}

impl AgentHandler {
    pub fn new(agent_service: Arc<dyn AgentSessionServiceInterface>) -> Self {
        Self { agent_service }
    }

    fn json_map(data: &Value) -> Option<&Map<String, Value>> {
        data.as_object()
    }

    fn get_str(data: &Map<String, Value>, key: &str) -> Option<String> {
        data.get(key)
            .and_then(|value| value.as_str())
            .map(str::to_string)
    }

    fn get_i64(data: &Map<String, Value>, key: &str) -> Option<i64> {
        data.get(key).and_then(|value| value.as_i64())
    }

    fn get_bool(data: &Map<String, Value>, key: &str) -> Option<bool> {
        data.get(key).and_then(|value| value.as_bool())
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<AgentArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;

        let data = match Self::json_map(&args.data) {
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
                let tool_name = match Self::get_str(data, "tool_name") {
                    Some(value) => value,
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing tool_name",
                        )]));
                    }
                };
                let tool_call = ToolCall {
                    id: format!("tc_{}", Uuid::new_v4()),
                    session_id: args.session_id.clone(),
                    tool_name: tool_name.clone(),
                    params_summary: Self::get_str(data, "params_summary"),
                    success: Self::get_bool(data, "success").unwrap_or(true),
                    error_message: Self::get_str(data, "error_message"),
                    duration_ms: Self::get_i64(data, "duration_ms"),
                    created_at: now,
                };
                match self.agent_service.store_tool_call(tool_call).await {
                    Ok(id) => ResponseFormatter::json_success(&serde_json::json!({
                        "tool_call_id": id,
                        "session_id": args.session_id,
                        "tool_name": tool_name,
                    })),
                    Err(_) => Ok(CallToolResult::error(vec![Content::text(
                        "Failed to store tool call",
                    )])),
                }
            }
            AgentAction::LogDelegation => {
                let child_session_id = match Self::get_str(data, "child_session_id") {
                    Some(value) => value,
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing child_session_id",
                        )]));
                    }
                };
                let delegation = Delegation {
                    id: format!("del_{}", Uuid::new_v4()),
                    parent_session_id: args.session_id.clone(),
                    child_session_id: child_session_id.clone(),
                    prompt: Self::get_str(data, "prompt").unwrap_or_default(),
                    prompt_embedding_id: Self::get_str(data, "prompt_embedding_id"),
                    result: Self::get_str(data, "result"),
                    success: Self::get_bool(data, "success").unwrap_or(true),
                    created_at: now,
                    completed_at: None,
                    duration_ms: Self::get_i64(data, "duration_ms"),
                };
                match self.agent_service.store_delegation(delegation).await {
                    Ok(id) => ResponseFormatter::json_success(&serde_json::json!({
                        "delegation_id": id,
                        "parent_session_id": args.session_id,
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
