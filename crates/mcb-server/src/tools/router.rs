//!
//! Routes incoming tool call requests to the appropriate handlers.
//! This module provides a centralized dispatch mechanism for MCP tool calls.

use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use tracing::warn;

use crate::args::{
    AgentArgs, IndexArgs, MemoryArgs, ProjectArgs, SearchArgs, SessionArgs, ValidateArgs, VcsArgs,
};
use crate::handlers::{
    AgentHandler, IndexHandler, MemoryHandler, ProjectHandler, SearchHandler, SessionHandler,
    ValidateHandler, VcsHandler,
};
use crate::hooks::{HookProcessor, PostToolUseContext};

/// Handler references for tool routing
#[derive(Clone)]
pub struct ToolHandlers {
    pub index: Arc<IndexHandler>,
    pub search: Arc<SearchHandler>,
    pub validate: Arc<ValidateHandler>,
    pub memory: Arc<MemoryHandler>,
    pub session: Arc<SessionHandler>,
    pub agent: Arc<AgentHandler>,
    pub project: Arc<ProjectHandler>,
    pub vcs: Arc<VcsHandler>,
    pub hook_processor: Arc<HookProcessor>,
}

/// Route a tool call request to the appropriate handler
///
/// Parses the request arguments and delegates to the matching handler.
/// After tool execution, automatically triggers PostToolUse hook for memory operations.
pub async fn route_tool_call(
    request: CallToolRequestParams,
    handlers: &ToolHandlers,
) -> Result<CallToolResult, McpError> {
    let tool_name = request.name.clone();

    let result = match request.name.as_ref() {
        "index" => {
            let args = parse_args::<IndexArgs>(&request)?;
            handlers.index.handle(Parameters(args)).await
        }
        "search" => {
            let args = parse_args::<SearchArgs>(&request)?;
            handlers.search.handle(Parameters(args)).await
        }
        "validate" => {
            let args = parse_args::<ValidateArgs>(&request)?;
            handlers.validate.handle(Parameters(args)).await
        }
        "memory" => {
            let args = parse_args::<MemoryArgs>(&request)?;
            handlers.memory.handle(Parameters(args)).await
        }
        "session" => {
            let args = parse_args::<SessionArgs>(&request)?;
            handlers.session.handle(Parameters(args)).await
        }
        "agent" => {
            let args = parse_args::<AgentArgs>(&request)?;
            handlers.agent.handle(Parameters(args)).await
        }
        "project" => {
            let args = parse_args::<ProjectArgs>(&request)?;
            handlers.project.handle(Parameters(args)).await
        }
        "vcs" => {
            let args = parse_args::<VcsArgs>(&request)?;
            handlers.vcs.handle(Parameters(args)).await
        }
        _ => Err(McpError::invalid_params(
            format!("Unknown tool: {}", request.name),
            None,
        )),
    }?;

    if let Err(e) = trigger_post_tool_use_hook(&tool_name, &result, &handlers.hook_processor).await
    {
        warn!("PostToolUse hook failed (non-fatal): {}", e);
    }

    Ok(result)
}

async fn trigger_post_tool_use_hook(
    tool_name: &str,
    result: &CallToolResult,
    hook_processor: &HookProcessor,
) -> Result<(), String> {
    let context = PostToolUseContext::new(tool_name.to_string(), result.clone());
    hook_processor
        .process_post_tool_use(context)
        .await
        .map_err(|e| e.to_string())
}

/// Parse request arguments into the expected type
fn parse_args<T: serde::de::DeserializeOwned>(
    request: &CallToolRequestParams,
) -> Result<T, McpError> {
    let args_value = serde_json::Value::Object(request.arguments.clone().unwrap_or_default());
    serde_json::from_value(args_value)
        .map_err(|e| McpError::invalid_params(format!("Failed to parse arguments: {e}"), None))
}
