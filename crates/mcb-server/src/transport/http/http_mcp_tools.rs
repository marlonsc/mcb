//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use std::sync::Arc;

use mcb_domain::error;
use rmcp::model::CallToolRequestParams;

use super::HttpTransportState;
use super::http_bridge::BridgeProvenance;
use crate::McpServer;
use crate::constants::{JSONRPC_INTERNAL_ERROR, JSONRPC_INVALID_PARAMS};
use crate::tools::{ToolExecutionContext, ToolHandlers, route_tool_call};
use crate::transport::types::{McpRequest, McpResponse};

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

pub(super) async fn handle_tools_call(
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

        // Env var fallbacks (mirrors stdio mode in mcp_server.rs)
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

        // VCS repo_id auto-derivation (mirrors mcp_server.rs)
        if let Some(ref path_str) = ctx.repo_path {
            if ctx.repo_id.as_deref().is_none_or(|s| s.trim().is_empty()) {
                if let Ok(repo) = state
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
            }
        }

        ctx
    };

    execution_context.apply_to_request_if_missing(&mut call_request);
    let handlers = build_tool_handlers(&state.server);

    match route_tool_call(call_request, &handlers, execution_context).await {
        Ok(result) => McpResponse::success(request.id.clone(), tool_result_to_json(&result)),
        Err(e) => {
            error!("HttpMcpTools", "Tool call failed", &e);
            let code = if e.code.0 == JSONRPC_INVALID_PARAMS {
                JSONRPC_INVALID_PARAMS
            } else {
                JSONRPC_INTERNAL_ERROR
            };
            McpResponse::error(request.id.clone(), code, format!("Tool call failed: {e:?}"))
        }
    }
}
