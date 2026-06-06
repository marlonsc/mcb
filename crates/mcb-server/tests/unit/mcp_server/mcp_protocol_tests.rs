//! MCP protocol compliance: JSON-RPC 2.0, initialize, tool schemas.
//!
//! Regression guards for protocol-level issues (version format leaks,
//! null schemas, missing jsonrpc field, broken error codes).

use rstest::rstest;

use axum::http::StatusCode;
use mcb_domain::protocol::McpRequest;
use mcb_utils::constants::protocol::JSONRPC_VERSION;

use crate::utils::http_mcp::{McpTestContext, post_mcp};

fn mcp_request(method: &str) -> McpRequest {
    McpRequest {
        jsonrpc: JSONRPC_VERSION.to_owned(),
        method: method.to_owned(),
        params: None,
        id: Some(serde_json::json!(1)),
    }
}

const HIDDEN_CONTEXT_FIELDS: &[&str] = &[
    "collection",
    "repo_id",
    "repo_path",
    "auth",
    "worktree_id",
    "agent_program",
    "model_id",
    "operator_id",
    "machine_id",
];

// ─── Initialize handshake ────────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn initialize_returns_valid_protocol_version() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let (status, resp) = post_mcp(&ctx, &mcp_request("initialize"), &[]).await?;
    assert_eq!(status, StatusCode::OK);
    assert!(resp.error.is_none(), "initialize must not error");

    let result = resp.result.ok_or("missing result")?;

    // Protocol version: must be a date string like "2024-11-05", not Debug format
    let version = result
        .get("protocolVersion")
        .and_then(|v| v.as_str())
        .ok_or("missing protocolVersion")?;
    assert!(
        version.contains('-') && !version.contains("ProtocolVersion"),
        "protocolVersion must be date-formatted, got: {version}"
    );

    // Server info
    let info = result.get("serverInfo").ok_or("missing serverInfo")?;
    assert!(
        info.get("name")
            .and_then(|n| n.as_str())
            .is_some_and(|n| !n.is_empty()),
        "serverInfo.name must be non-empty string"
    );
    assert!(
        info.get("version")
            .is_some_and(serde_json::Value::is_string),
        "serverInfo.version must be string"
    );

    // Capabilities
    let caps = result.get("capabilities").ok_or("missing capabilities")?;
    assert!(
        caps.get("tools").is_some_and(serde_json::Value::is_object),
        "capabilities.tools must be object"
    );

    Ok(())
}

// ─── Tool schema validation ──────────────────────────────────────────

#[rstest]
#[tokio::test]
async fn all_tools_have_valid_object_schemas() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let (_, resp) = post_mcp(&ctx, &mcp_request("tools/list"), &[]).await?;
    let tools = resp
        .result
        .as_ref()
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .ok_or("tools array missing")?;

    assert!(!tools.is_empty());

    for tool in tools {
        let name = tool.get("name").and_then(|n| n.as_str()).unwrap_or("?");
        let schema = tool.get("inputSchema").ok_or("missing inputSchema")?;

        assert!(!schema.is_null(), "'{name}' has null inputSchema");
        assert!(schema.is_object(), "'{name}' inputSchema must be object");
        assert_eq!(
            schema.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "'{name}' schema type must be object"
        );
    }

    Ok(())
}

#[rstest]
#[tokio::test]
async fn search_code_requires_query_parameter() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let (_, resp) = post_mcp(&ctx, &mcp_request("tools/list"), &[]).await?;
    let tools = resp
        .result
        .as_ref()
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .ok_or("tools array")?;

    let search = tools
        .iter()
        .find(|t| t.get("name").and_then(|n| n.as_str()) == Some("search_code"))
        .ok_or("search_code tool missing")?;

    let required = search
        .get("inputSchema")
        .and_then(|s| s.get("required"))
        .and_then(|r| r.as_array())
        .ok_or("required array")?;

    assert!(
        required.iter().any(|v| v.as_str() == Some("query")),
        "search_code must require 'query'"
    );

    Ok(())
}

#[rstest]
#[tokio::test]
async fn context_fields_hidden_from_all_tool_schemas() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let (_, resp) = post_mcp(&ctx, &mcp_request("tools/list"), &[]).await?;
    let tools = resp
        .result
        .as_ref()
        .and_then(|r| r.get("tools"))
        .and_then(|t| t.as_array())
        .ok_or("tools array")?;

    for tool in tools {
        let name = tool.get("name").and_then(|n| n.as_str()).unwrap_or("?");
        if let Some(props) = tool
            .get("inputSchema")
            .and_then(|s| s.get("properties"))
            .and_then(|p| p.as_object())
        {
            for field in HIDDEN_CONTEXT_FIELDS {
                assert!(
                    !props.contains_key(*field),
                    "'{name}' exposes hidden field '{field}'"
                );
            }
        }
    }

    Ok(())
}

// ─── JSON-RPC 2.0 format ────────────────────────────────────────────

#[rstest]
#[case("initialize")]
#[case("tools/list")]
#[case("ping")]
#[tokio::test]
async fn every_response_includes_jsonrpc_version(
    #[case] method: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let (_, resp) = post_mcp(&ctx, &mcp_request(method), &[]).await?;
    assert_eq!(resp.jsonrpc, JSONRPC_VERSION);
    Ok(())
}

#[rstest]
#[case(serde_json::json!(42))]
#[case(serde_json::json!("test-id-123"))]
#[tokio::test]
async fn response_echoes_request_id(
    #[case] id: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let mut req = mcp_request("ping");
    req.id = Some(id.clone());
    let (_, resp) = post_mcp(&ctx, &req, &[]).await?;
    assert_eq!(resp.id, Some(id));
    Ok(())
}

#[rstest]
#[tokio::test]
async fn unknown_method_returns_error_32601() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let (_, resp) = post_mcp(&ctx, &mcp_request("nonexistent/method"), &[]).await?;

    let err = resp.error.ok_or("expected error")?;
    assert_eq!(err.code, -32601);
    assert!(!err.message.is_empty());
    Ok(())
}
