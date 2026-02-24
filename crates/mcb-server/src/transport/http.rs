use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use rmcp::ServerHandler;
use rmcp::model::CallToolRequestParams;

use crate::McpServer;
use crate::constants::{JSONRPC_INTERNAL_ERROR, JSONRPC_INVALID_PARAMS, JSONRPC_METHOD_NOT_FOUND};
use crate::tools::{ToolExecutionContext, route_tool_call};
use crate::transport::types::{McpRequest, McpResponse};

/// Shared state for the HTTP transport layer.
#[derive(Clone)]
pub struct HttpTransportState {
    /// The MCP server instance used to handle incoming requests.
    pub server: Arc<McpServer>,
}

/// Handles an incoming MCP JSON-RPC request over HTTP.
pub async fn handle_mcp_request(
    State(state): State<Arc<HttpTransportState>>,
    headers: HeaderMap,
    Json(request): Json<McpRequest>,
) -> Json<McpResponse> {
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(&state, &request).await,
        "tools/list" => handle_tools_list(&request).await,
        "tools/call" => handle_tools_call(&state, &headers, &request).await,
        "ping" => McpResponse::success(request.id.clone(), serde_json::json!({})),
        _ => McpResponse::error(
            request.id.clone(),
            JSONRPC_METHOD_NOT_FOUND,
            format!("Unknown method: {}", request.method),
        ),
    };

    Json(response)
}

async fn handle_initialize(state: &HttpTransportState, request: &McpRequest) -> McpResponse {
    let server_info = state.server.get_info();
    let result = serde_json::json!({
        "protocolVersion": server_info.protocol_version.to_string(),
        "capabilities": { "tools": {} },
        "serverInfo": {
            "name": server_info.server_info.name,
            "version": server_info.server_info.version
        },
        "instructions": server_info.instructions
    });
    McpResponse::success(request.id.clone(), result)
}

async fn handle_tools_list(request: &McpRequest) -> McpResponse {
    match crate::tools::create_tool_list() {
        Ok(tools) => {
            let tools_json: Vec<serde_json::Value> = tools
                .into_iter()
                .map(|tool| {
                    serde_json::json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": serde_json::to_value(tool.input_schema.as_ref()).ok()
                    })
                })
                .collect();

            McpResponse::success(
                request.id.clone(),
                serde_json::json!({ "tools": tools_json }),
            )
        }
        Err(e) => {
            mcb_domain::error!("HttpTransport", "Failed to list tools", &e);
            McpResponse::error(
                request.id.clone(),
                JSONRPC_INTERNAL_ERROR,
                format!("Failed to list tools: {e:?}"),
            )
        }
    }
}

fn parse_tool_call_params(
    params: &serde_json::Value,
) -> Result<CallToolRequestParams, (i32, &'static str)> {
    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or((
            JSONRPC_INVALID_PARAMS,
            "Missing 'name' parameter for tools/call",
        ))?
        .to_owned();

    let arguments = match params.get("arguments") {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => {
            let object = value.as_object().cloned().ok_or((
                JSONRPC_INVALID_PARAMS,
                "Invalid 'arguments' parameter for tools/call: expected object",
            ))?;
            Some(object)
        }
    };

    Ok(CallToolRequestParams {
        name: tool_name.into(),
        arguments,
        task: None,
        meta: None,
    })
}

fn tool_result_to_json(result: &rmcp::model::CallToolResult) -> serde_json::Value {
    let content_json: Vec<serde_json::Value> = result
        .content
        .iter()
        .map(|content| match serde_json::to_value(content) {
            Ok(value) => value,
            Err(e) => serde_json::json!({
                "type": "text",
                "text": format!("Error serializing content: {}", e)
            }),
        })
        .collect();

    serde_json::json!({
        "content": content_json,
        "isError": result.is_error.unwrap_or(false)
    })
}

fn extract_override(headers: &HeaderMap, header_name: &str) -> Option<String> {
    headers
        .get(header_name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn build_overrides(headers: &HeaderMap) -> HashMap<String, String> {
    let mut overrides = HashMap::new();
    let mappings = [
        ("X-Workspace-Root", "workspace_root"),
        ("X-Repo-Path", "repo_path"),
        ("X-Repo-Id", "repo_id"),
        ("X-Session-Id", "session_id"),
        ("X-Parent-Session-Id", "parent_session_id"),
        ("X-Project-Id", "project_id"),
        ("X-Worktree-Id", "worktree_id"),
        ("X-Operator-Id", "operator_id"),
        ("X-Machine-Id", "machine_id"),
        ("X-Agent-Program", "agent_program"),
        ("X-Model-Id", "model_id"),
        ("X-Delegated", "delegated"),
        ("X-Execution-Flow", "execution_flow"),
    ];

    for (header_name, key) in mappings {
        if let Some(value) = extract_override(headers, header_name) {
            overrides.insert(key.to_owned(), value);
        }
    }

    overrides
}

async fn handle_tools_call(
    state: &HttpTransportState,
    headers: &HeaderMap,
    request: &McpRequest,
) -> McpResponse {
    let overrides = build_overrides(headers);
    let has_workspace_provenance = overrides
        .get("workspace_root")
        .is_some_and(|value| !value.trim().is_empty())
        || overrides
            .get("repo_path")
            .is_some_and(|value| !value.trim().is_empty());

    if !has_workspace_provenance {
        return McpResponse::error(
            request.id.clone(),
            JSONRPC_INVALID_PARAMS,
            "Direct HTTP tools/call is not supported. Use stdio or stdio bridge and provide workspace provenance headers.",
        );
    }

    let params = match &request.params {
        Some(params) => params,
        None => {
            return McpResponse::error(
                request.id.clone(),
                JSONRPC_INVALID_PARAMS,
                "Missing params for tools/call",
            );
        }
    };

    let mut call_request = match parse_tool_call_params(params) {
        Ok(req) => req,
        Err((code, msg)) => return McpResponse::error(request.id.clone(), code, msg),
    };

    let mut execution_context =
        ToolExecutionContext::resolve(&state.server.runtime_defaults(), &overrides);

    if execution_context
        .agent_program
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        execution_context.agent_program = Some("mcb-http-bridge".to_owned());
    }
    if execution_context
        .model_id
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        execution_context.model_id = Some("unknown".to_owned());
    }
    if execution_context
        .execution_flow
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        execution_context.execution_flow = Some("server-hybrid".to_owned());
    }

    if let Some(path_str) = execution_context.repo_path.as_deref()
        && execution_context
            .repo_id
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        && let Ok(repo) = state
            .server
            .vcs_provider()
            .open_repository(Path::new(path_str))
            .await
    {
        execution_context.repo_id = Some(
            state
                .server
                .vcs_provider()
                .repository_id(&repo)
                .into_string(),
        );
    }

    execution_context.apply_to_request_if_missing(&mut call_request);
    let handlers = state.server.tool_handlers();

    match route_tool_call(call_request, &handlers, execution_context).await {
        Ok(result) => McpResponse::success(request.id.clone(), tool_result_to_json(&result)),
        Err(e) => {
            mcb_domain::error!("HttpTransport", "Tool call failed", &e);
            let code = if e.code.0 == JSONRPC_INVALID_PARAMS {
                JSONRPC_INVALID_PARAMS
            } else {
                JSONRPC_INTERNAL_ERROR
            };
            McpResponse::error(request.id.clone(), code, format!("Tool call failed: {e:?}"))
        }
    }
}
