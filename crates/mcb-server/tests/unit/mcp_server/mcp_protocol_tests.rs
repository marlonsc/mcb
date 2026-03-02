//! MCP Protocol Compliance Tests
//!
//! Tests that validate MCP protocol compliance to prevent regressions.
//! These tests specifically target issues fixed in commits ffbe441 and a1af74c:
//! - Protocol version serialization format
//! - Tool inputSchema presence and validity
//! - JSON-RPC 2.0 response format
//!
//! Run with: `cargo test -p mcb-server --test unit mcp_protocol`

use rstest::rstest;

use axum::http::StatusCode;
use mcb_server::transport::types::McpRequest;

use crate::utils::http_mcp::{McpTestContext, post_mcp};

// =============================================================================
// PROTOCOL VERSION & INITIALIZE TESTS
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_initialize_response() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let request = McpRequest {
        method: "initialize".to_owned(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let (status, mcp_response) = post_mcp(&ctx, &request, &[]).await?;
    assert_eq!(status, StatusCode::OK);

    assert!(mcp_response.error.is_none(), "Initialize should not error");

    let result_opt = mcp_response.result;
    assert!(result_opt.is_some(), "Should have result");
    let result = match result_opt {
        Some(value) => value,
        None => return Ok(()),
    };

    // Check protocol version (regression test for a1af74c)
    let version_opt = result.get("protocolVersion");
    assert!(version_opt.is_some(), "Should have protocolVersion");
    let version = match version_opt {
        Some(value) => value,
        None => return Ok(()),
    };

    assert!(
        version.is_string(),
        "protocolVersion must be a JSON string. Got: {version:?}"
    );
    let version_opt = version.as_str();
    assert!(version_opt.is_some(), "version must be string");
    let version_str = match version_opt {
        Some(value) => value,
        None => return Ok(()),
    };
    assert!(
        !version_str.contains("ProtocolVersion"),
        "protocolVersion has Debug format leak: {version_str}"
    );
    assert!(
        version_str.contains("-"),
        "protocolVersion should be date-formatted: {version_str}"
    );

    // Check serverInfo
    let server_info_opt = result.get("serverInfo");
    assert!(server_info_opt.is_some(), "Should have serverInfo");
    let server_info = match server_info_opt {
        Some(value) => value,
        None => return Ok(()),
    };
    assert!(server_info.is_object(), "serverInfo should be an object");
    let name_opt = server_info.get("name");
    assert!(name_opt.is_some(), "Should have name");
    let name = match name_opt {
        Some(value) => value,
        None => return Ok(()),
    };
    assert!(
        name.is_string() && name.as_str().is_some_and(|value| !value.is_empty()),
        "Invalid name"
    );
    assert!(
        server_info
            .get("version")
            .is_some_and(serde_json::Value::is_string),
        "Invalid version"
    );

    // Check capabilities
    let capabilities_opt = result.get("capabilities");
    assert!(capabilities_opt.is_some(), "Should have capabilities");
    let capabilities = match capabilities_opt {
        Some(value) => value,
        None => return Ok(()),
    };
    assert!(capabilities.is_object(), "capabilities should be an object");
    assert!(
        capabilities
            .get("tools")
            .is_some_and(serde_json::Value::is_object),
        "tools cap should be object"
    );
    Ok(())
}

// =============================================================================
// TOOL SCHEMA VALIDATION TESTS
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_tools_schemas() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let request = McpRequest {
        method: "tools/list".to_owned(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let (_, mcp_response) = post_mcp(&ctx, &request, &[]).await?;
    let result_opt = mcp_response.result;
    assert!(result_opt.is_some(), "Should have result");
    let result = match result_opt {
        Some(value) => value,
        None => return Ok(()),
    };

    let tools_opt = result.get("tools");
    assert!(tools_opt.is_some(), "Should have tools array");
    let tools = match tools_opt {
        Some(value) => value,
        None => return Ok(()),
    };
    let tools_array_opt = tools.as_array();
    assert!(tools_array_opt.is_some(), "tools should be array");
    let tools_array = match tools_array_opt {
        Some(value) => value,
        None => return Ok(()),
    };
    assert!(!tools_array.is_empty(), "Should have at least one tool");

    // Verify all tools have valid schemas
    for tool in tools_array {
        let tool_name = tool
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");
        let schema_opt = tool.get("inputSchema");
        assert!(schema_opt.is_some(), "Missing inputSchema");
        let schema = match schema_opt {
            Some(value) => value,
            None => continue,
        };

        // Regression check: not null
        assert!(!schema.is_null(), "Tool '{tool_name}' has null inputSchema");
        assert!(
            schema.is_object(),
            "Tool '{tool_name}' inputSchema should be object"
        );

        let schema_type_opt = schema.get("type");
        assert!(schema_type_opt.is_some(), "Missing type in schema");
        let schema_type = match schema_type_opt {
            Some(value) => value,
            None => continue,
        };
        assert_eq!(
            schema_type.as_str(),
            Some("object"),
            "Schema type must be object"
        );

        assert!(
            schema
                .get("properties")
                .is_some_and(serde_json::Value::is_object),
            "properties must be object"
        );
    }

    // Verify specific tools requirements
    let index_tool = tools_array
        .iter()
        .find(|t| t.get("name").and_then(|n| n.as_str()) == Some("index"));
    assert!(index_tool.is_some(), "index tool missing");
    let index_tool = match index_tool {
        Some(value) => value,
        None => return Ok(()),
    };
    let required = index_tool
        .get("inputSchema")
        .and_then(|v| v.get("required"))
        .and_then(serde_json::Value::as_array);
    assert!(required.is_some(), "req array");
    let required = match required {
        Some(value) => value,
        None => return Ok(()),
    };
    assert!(
        required.iter().any(|v| v.as_str() == Some("action")),
        "index tool must require action"
    );

    let search_tool = tools_array
        .iter()
        .find(|t| t.get("name").and_then(|n| n.as_str()) == Some("search"));
    assert!(search_tool.is_some(), "search tool missing");
    let search_tool = match search_tool {
        Some(value) => value,
        None => return Ok(()),
    };
    let required = search_tool
        .get("inputSchema")
        .and_then(|v| v.get("required"))
        .and_then(serde_json::Value::as_array);
    assert!(required.is_some(), "req array");
    let required = match required {
        Some(value) => value,
        None => return Ok(()),
    };
    assert!(
        required.iter().any(|v| v.as_str() == Some("query")),
        "search tool must require query"
    );
    Ok(())
}

// =============================================================================
// JSON-RPC 2.0 FORMAT TESTS
// =============================================================================

#[rstest]
#[case("initialize")]
#[case("tools/list")]
#[case("ping")]
#[rstest]
#[tokio::test]
async fn test_response_has_jsonrpc_field(
    #[case] method: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let request = McpRequest {
        method: method.to_owned(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let (_, mcp_response) = post_mcp(&ctx, &request, &[]).await?;

    assert_eq!(
        mcp_response.jsonrpc, "2.0",
        "Response for '{method}' should have jsonrpc: \"2.0\""
    );
    Ok(())
}

#[rstest]
#[case(serde_json::json!(42))]
#[case(serde_json::json!("test-id-123"))]
#[tokio::test]
async fn test_response_echoes_request_id(
    #[case] id: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let request = McpRequest {
        method: "ping".to_owned(),
        params: None,
        id: Some(id.clone()),
    };

    let (_, mcp_response) = post_mcp(&ctx, &request, &[]).await?;

    assert_eq!(mcp_response.id, Some(id));
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_error_response_structure() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = McpTestContext::new().await?;
    let request = McpRequest {
        method: "nonexistent/method".to_owned(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let (_, mcp_response) = post_mcp(&ctx, &request, &[]).await?;

    assert!(mcp_response.error.is_some(), "Should have error");
    let error = match mcp_response.error {
        Some(error) => error,
        None => return Ok(()),
    };
    assert_eq!(error.code, -32601, "Error code -32601");
    assert!(!error.message.is_empty(), "Error message not empty");
    Ok(())
}
