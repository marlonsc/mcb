use std::sync::Arc;

use axum::http::StatusCode;
use mcb_server::McpServer;
use mcb_server::transport::types::{McpRequest, McpResponse};
use rmcp::ServerHandler;
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
    // Call the server directly using ServerHandler trait
    let response = ctx.server.handle_call(request.clone()).await;

    // Convert rmcp response to McpResponse
    let mcp_response = match response {
        Ok(result) => McpResponse {
            result: Some(result),
            error: None,
        },
        Err(err) => McpResponse {
            result: None,
            error: Some(serde_json::json!({
                "code": err.code,
                "message": err.message,
            })),
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
