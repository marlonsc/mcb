use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use rmcp::ServerHandler;
use rmcp::model::CallToolRequestParams;

use crate::McpServer;
use crate::constants::{JSONRPC_INTERNAL_ERROR, JSONRPC_INVALID_PARAMS, JSONRPC_METHOD_NOT_FOUND};
use crate::tools::{ToolExecutionContext, ToolHandlers, route_tool_call};
use crate::transport::types::{McpRequest, McpResponse};

#[derive(Clone)]
struct BridgeProvenance {
    workspace_root: Option<String>,
    repo_path: Option<String>,
    repo_id: Option<String>,
    session_id: Option<String>,
    parent_session_id: Option<String>,
    project_id: Option<String>,
    worktree_id: Option<String>,
    operator_id: Option<String>,
    machine_id: Option<String>,
    agent_program: Option<String>,
    model_id: Option<String>,
    delegated: Option<String>,
    execution_flow: Option<String>,
}

impl BridgeProvenance {
    fn from_headers(headers: &HeaderMap) -> Self {
        let header = |name: &str| {
            headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned)
        };

        Self {
            workspace_root: header("X-Workspace-Root"),
            repo_path: header("X-Repo-Path"),
            repo_id: header("X-Repo-Id"),
            session_id: header("X-Session-Id"),
            parent_session_id: header("X-Parent-Session-Id"),
            project_id: header("X-Project-Id"),
            worktree_id: header("X-Worktree-Id"),
            operator_id: header("X-Operator-Id"),
            machine_id: header("X-Machine-Id"),
            agent_program: header("X-Agent-Program"),
            model_id: header("X-Model-Id"),
            delegated: header("X-Delegated"),
            execution_flow: header("X-Execution-Flow"),
        }
    }
}

#[derive(Clone)]
pub struct HttpTransportState {
    pub server: Arc<McpServer>,
}

pub async fn handle_mcp_request(
    State(state): State<Arc<HttpTransportState>>,
    headers: HeaderMap,
    Json(request): Json<McpRequest>,
) -> Json<McpResponse> {
    let provenance = BridgeProvenance::from_headers(&headers);
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(&state, &request).await,
        "tools/list" => handle_tools_list(&request).await,
        "tools/call" => handle_tools_call(&state, &provenance, &request).await,
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

fn parse_delegated_flag(raw: Option<&str>) -> Option<bool> {
    raw.map(str::trim)
        .and_then(|v| match v.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" => Some(false),
            _ => None,
        })
}

fn build_tool_handlers(server: &Arc<McpServer>) -> ToolHandlers {
    server.tool_handlers()
}

async fn handle_tools_call(
    state: &HttpTransportState,
    bridge_provenance: &BridgeProvenance,
    request: &McpRequest,
) -> McpResponse {
    let has_workspace_provenance = bridge_provenance
        .workspace_root
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || bridge_provenance
            .repo_path
            .as_deref()
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

    let execution_context = {
        let mut ctx = ToolExecutionContext {
            session_id: bridge_provenance.session_id.clone(),
            parent_session_id: bridge_provenance.parent_session_id.clone(),
            project_id: bridge_provenance.project_id.clone(),
            worktree_id: bridge_provenance.worktree_id.clone(),
            repo_id: bridge_provenance.repo_id.clone(),
            repo_path: bridge_provenance
                .repo_path
                .clone()
                .or_else(|| bridge_provenance.workspace_root.clone()),
            operator_id: bridge_provenance.operator_id.clone(),
            machine_id: bridge_provenance.machine_id.clone(),
            agent_program: bridge_provenance.agent_program.clone(),
            model_id: bridge_provenance.model_id.clone(),
            delegated: parse_delegated_flag(bridge_provenance.delegated.as_deref()),
            timestamp: Some(chrono::Utc::now().timestamp()),
            execution_flow: bridge_provenance
                .execution_flow
                .clone()
                .or_else(|| Some("server-hybrid".to_owned())),
        };

        if ctx
            .operator_id
            .as_deref()
            .is_none_or(|s| s.trim().is_empty())
        {
            ctx.operator_id = std::env::var("USER").ok();
        }
        if ctx
            .machine_id
            .as_deref()
            .is_none_or(|s| s.trim().is_empty())
        {
            ctx.machine_id = std::env::var("HOSTNAME").ok();
        }
        if ctx
            .agent_program
            .as_deref()
            .is_none_or(|s| s.trim().is_empty())
        {
            ctx.agent_program = Some("mcb-http-bridge".to_owned());
        }
        if ctx.model_id.as_deref().is_none_or(|s| s.trim().is_empty()) {
            ctx.model_id = Some("unknown".to_owned());
        }
        if ctx.delegated.is_none() {
            ctx.delegated = Some(ctx.parent_session_id.is_some());
        }

        if let Some(ref path_str) = ctx.repo_path
            && ctx.repo_id.as_deref().is_none_or(|s| s.trim().is_empty())
            && let Ok(repo) = state
                .server
                .vcs_provider()
                .open_repository(std::path::Path::new(path_str))
                .await
        {
            ctx.repo_id = Some(
                state
                    .server
                    .vcs_provider()
                    .repository_id(&repo)
                    .into_string(),
            );
        }

        ctx
    };

    execution_context.apply_to_request_if_missing(&mut call_request);
    let handlers = build_tool_handlers(&state.server);

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
