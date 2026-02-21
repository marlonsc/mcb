//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use mcb_domain::error;
use rmcp::ServerHandler;
use rocket::serde::json::Json;
use rocket::{State, post};

use super::HttpTransportState;
use super::http_bridge::BridgeProvenance;
use super::http_mcp_tools::handle_tools_call;
use crate::constants::{JSONRPC_INTERNAL_ERROR, JSONRPC_METHOD_NOT_FOUND};
use crate::transport::types::{McpRequest, McpResponse};

#[post("/mcp", format = "json", data = "<request>")]
pub(super) async fn handle_mcp_request(
    state: &State<HttpTransportState>,
    bridge_provenance: BridgeProvenance,
    request: Json<McpRequest>,
) -> Json<McpResponse> {
    let request = request.into_inner();
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(state, &request).await,
        "tools/list" => handle_tools_list(&request).await,
        "tools/call" => handle_tools_call(state, &bridge_provenance, &request).await,
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
            error!("HttpMcp", "Failed to list tools", &e);
            McpResponse::error(
                request.id.clone(),
                JSONRPC_INTERNAL_ERROR,
                format!("Failed to list tools: {e:?}"),
            )
        }
    }
}
