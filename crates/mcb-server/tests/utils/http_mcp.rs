use std::sync::Arc;

use axum::http::StatusCode;
use mcb_server::McpServer;
use mcb_server::tools::router::{ToolExecutionContext, route_tool_call};
use mcb_server::transport::types::{McpError, McpRequest, McpResponse};
use rmcp::model::CallToolRequestParams;
use tempfile::TempDir;

use crate::utils::test_fixtures::create_test_mcp_server;

pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub struct McpTestContext {
    pub server: Arc<McpServer>,
    pub _temp: TempDir,
}

impl McpTestContext {
    pub async fn new() -> TestResult<Self> {
        let (server_instance, temp) = create_test_mcp_server().await;
        let server = Arc::new(server_instance);

        Ok(Self {
            server,
            _temp: temp,
        })
    }
}

pub async fn post_mcp(
    ctx: &McpTestContext,
    request: &McpRequest,
    _headers: &[(String, String)],
) -> TestResult<(StatusCode, McpResponse)> {
    // Handle tools/call method
    if request.method != "tools/call" {
        let mcp_response = McpResponse {
            jsonrpc: "2.0".to_owned(),
            result: None,
            error: Some(McpError {
                code: -32601,
                message: format!("Unknown tool: {}", request.method),
            }),
            id: request.id.clone(),
        };
        return Ok((StatusCode::OK, mcp_response));
    }

    // Extract tool name and arguments from params
    let params = request
        .params
        .as_ref()
        .ok_or("Missing params for tools/call")?;
    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("Missing tool name in params")?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    // Convert to CallToolRequestParams
    let call_request = CallToolRequestParams {
        name: tool_name.to_owned().into(),
        arguments: if let serde_json::Value::Object(map) = arguments {
            Some(map)
        } else {
            Some(serde_json::Map::new())
        },
        task: None,
        meta: None,
    };

    // Call the server using route_tool_call
    let response = route_tool_call(
        call_request,
        &ctx.server.tool_handlers(),
        ToolExecutionContext::default(),
    )
    .await;

    // Convert rmcp response to McpResponse
    let mcp_response = match response {
        Ok(result) => McpResponse {
            jsonrpc: "2.0".to_owned(),
            result: Some(serde_json::to_value(&result)?),
            error: None,
            id: request.id.clone(),
        },
        Err(err) => McpResponse {
            jsonrpc: "2.0".to_owned(),
            result: None,
            error: Some(McpError {
                code: err.code.0,
                message: err.message.to_string(),
            }),
            id: request.id.clone(),
        },
    };

    Ok((StatusCode::OK, mcp_response))
}

/// Helper for tests using static string slices
pub async fn post_mcp_str(
    ctx: &McpTestContext,
    request: &McpRequest,
    headers: &[(&str, &str)],
) -> TestResult<(StatusCode, McpResponse)> {
    let owned_headers: Vec<(String, String)> = headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    post_mcp(ctx, request, &owned_headers).await
}

pub fn tools_list_request() -> McpRequest {
    McpRequest {
        method: "tools/list".to_owned(),
        params: None,
        id: Some(serde_json::json!(1)),
    }
}

pub fn tools_call_request(tool_name: &str) -> McpRequest {
    McpRequest {
        method: "tools/call".to_owned(),
        params: Some(serde_json::json!({
            "name": tool_name,
            "arguments": {}
        })),
        id: Some(serde_json::json!(1)),
    }
}
