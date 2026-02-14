//! Agent handler for tool call and delegation logging.

use std::sync::Arc;

use mcb_domain::entities::agent::{Delegation, ToolCall};
use mcb_domain::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde_json::Value;

use uuid::Uuid;
use validator::Validate;

use crate::args::{AgentAction, AgentArgs};
use crate::formatter::ResponseFormatter;
use crate::handlers::helpers::resolve_org_id;

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
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<AgentArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("invalid arguments", None))?;

        let _org_id = resolve_org_id(args.org_id.as_deref());

        let session_id = args.session_id.as_str();
        if session_id.is_empty()
            || args.session_id.inner() == uuid::Uuid::nil()
            || args.session_id == mcb_domain::value_objects::SessionId::from_name("")
        {
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
        let now = mcb_domain::utils::time::epoch_secs_i64()
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        match args.action {
            AgentAction::LogTool => {
                let tool_name = match data
                    .get("tool_name")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                {
                    Some(value) => value,
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing tool_name",
                        )]));
                    }
                };
                let tool_call = ToolCall {
                    id: format!("tc_{}", Uuid::new_v4()),
                    session_id: session_id.to_owned(),
                    tool_name: tool_name.clone(),
                    params_summary: data
                        .get("params_summary")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    success: data.get("success").and_then(Value::as_bool).unwrap_or(true),
                    error_message: data
                        .get("error_message")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    duration_ms: data.get("duration_ms").and_then(Value::as_i64),
                    created_at: now,
                };
                match self.agent_service.store_tool_call(tool_call).await {
                    Ok(id) => ResponseFormatter::json_success(&serde_json::json!({
                        "tool_call_id": id,
                        "session_id": session_id,
                        "tool_name": tool_name,
                    })),
                    Err(_) => Ok(CallToolResult::error(vec![Content::text(
                        "Failed to store tool call",
                    )])),
                }
            }
            AgentAction::LogDelegation => {
                let child_session_id = match data
                    .get("child_session_id")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                {
                    Some(value) => value,
                    None => {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "Missing child_session_id",
                        )]));
                    }
                };
                let delegation = Delegation {
                    id: format!("del_{}", Uuid::new_v4()),
                    parent_session_id: session_id.to_owned(),
                    child_session_id: child_session_id.clone(),
                    prompt: data
                        .get("prompt")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                        .unwrap_or_default(),
                    prompt_embedding_id: data
                        .get("prompt_embedding_id")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    result: data
                        .get("result")
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    success: data.get("success").and_then(Value::as_bool).unwrap_or(true),
                    created_at: now,
                    completed_at: None,
                    duration_ms: data.get("duration_ms").and_then(Value::as_i64),
                };
                match self.agent_service.store_delegation(delegation).await {
                    Ok(id) => ResponseFormatter::json_success(&serde_json::json!({
                        "delegation_id": id,
                        "parent_session_id": session_id,
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
