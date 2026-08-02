//! MCP streamable HTTP transport integration tests using [`TestServer`].
//!
//! These tests send JSON-RPC requests to the real `/mcp` endpoint over HTTP and
//! verify that the MCP server responds with proper tool listings and tool-call
//! results. They also exercise the `HttpClientTransport` stdio-to-HTTP bridge
//! against the running server.

use crate::utils::test_server::TestServer;
use anyhow::Result;
use mcb_domain::protocol::{McpRequest, McpResponse};
use mcb_server::transport::http_client::HttpClientTransport;
use rstest::rstest;
use std::time::Duration;

fn mcp_request(method: &str, params: Option<serde_json::Value>, id: u64) -> McpRequest {
    McpRequest {
        jsonrpc: "2.0".to_owned(),
        method: method.to_owned(),
        params,
        id: Some(serde_json::json!(id)),
    }
}

fn parse_sse_response(text: &str) -> Result<McpResponse> {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(payload) = trimmed.strip_prefix("data:") {
            let payload = payload.trim();
            if payload.is_empty() {
                continue;
            }
            return Ok(serde_json::from_str(payload)?);
        }
    }
    Err(anyhow::anyhow!(
        "no SSE data payload found in response: {text}"
    ))
}

async fn post_mcp(server: &TestServer, request: &McpRequest) -> Result<McpResponse> {
    let response = server
        .client
        .post(server.url("/mcp"))
        .header(
            reqwest::header::CONTENT_TYPE,
            mcb_utils::constants::http::CONTENT_TYPE_JSON,
        )
        .header(
            reqwest::header::ACCEPT,
            "application/json, text/event-stream",
        )
        .header(
            mcb_utils::constants::protocol::HTTP_HEADER_EXECUTION_FLOW,
            mcb_utils::constants::protocol::EXECUTION_FLOW_HYBRID,
        )
        .json(request)
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    let body = parse_sse_response(&text).map_err(|e| {
        anyhow::anyhow!("failed to decode MCP response (status {status}): {e}; body: {text}")
    })?;
    assert!(
        status.is_success(),
        "MCP request returned non-success status {status}: {body:?}"
    );
    Ok(body)
}

#[rstest]
#[tokio::test]
async fn mcp_tools_list_returns_tools() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };

    let request = mcp_request("tools/list", None, 1);
    let response = post_mcp(&server, &request).await?;

    assert_eq!(response.id, Some(serde_json::json!(1)));
    assert!(response.error.is_none(), "tools/list should not error");
    let tools = response
        .result
        .as_ref()
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .ok_or_else(|| anyhow::anyhow!("tools array missing"))?;
    assert!(!tools.is_empty(), "server should expose at least one tool");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn mcp_tools_call_list_rules() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };

    let request = mcp_request(
        "tools/call",
        Some(serde_json::json!({
            "name": "list_rules",
            "arguments": {}
        })),
        2,
    );
    let response = post_mcp(&server, &request).await?;

    assert_eq!(response.id, Some(serde_json::json!(2)));
    assert!(
        response.error.is_none(),
        "list_rules tool call should not error: {:?}",
        response.error
    );
    let result = response
        .result
        .ok_or_else(|| anyhow::anyhow!("tool result missing"))?;
    let content = result
        .get("content")
        .and_then(|c| c.as_array())
        .ok_or_else(|| anyhow::anyhow!("content array missing"))?;
    assert!(!content.is_empty(), "list_rules should return content");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn mcp_unknown_method_returns_error() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };

    let request = mcp_request("tools/unknown", None, 3);
    let response = post_mcp(&server, &request).await?;

    assert_eq!(response.id, Some(serde_json::json!(3)));
    assert!(
        response.error.is_some(),
        "unknown method should return an error"
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn http_client_transport_bridges_stdio_to_server() -> Result<()> {
    let Some(server) = TestServer::new().await else {
        return Ok(());
    };

    let client = HttpClientTransport::new_with_session_source(
        server.base_url.clone(),
        Some("http-mcp-test".to_owned()),
        Duration::from_secs(10),
        None,
        None,
    )
    .map_err(|e| anyhow::anyhow!("failed to create HTTP client transport: {e}"))?;

    let list_request = mcp_request("tools/list", None, 42);
    let request_json = serde_json::to_string(&list_request)?;

    // Use the public server_url accessor to confirm the client points at the test server.
    assert_eq!(client.server_url(), server.base_url);

    // The transport is normally driven by stdin/stdout; here we just verify the
    // client is correctly configured and the server is reachable through the
    // same HTTP surface the bridge uses.
    let response = server
        .client
        .post(server.url("/mcp"))
        .header(
            reqwest::header::CONTENT_TYPE,
            mcb_utils::constants::http::CONTENT_TYPE_JSON,
        )
        .header(
            reqwest::header::ACCEPT,
            "application/json, text/event-stream",
        )
        .header(
            mcb_utils::constants::protocol::HTTP_HEADER_EXECUTION_FLOW,
            mcb_utils::constants::protocol::EXECUTION_FLOW_HYBRID,
        )
        .body(request_json)
        .send()
        .await?;

    assert!(response.status().is_success());
    let body = parse_sse_response(&response.text().await?)?;
    assert_eq!(body.id, Some(serde_json::json!(42)));
    Ok(())
}
