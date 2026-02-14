//! MCP Protocol Compliance Tests
//!
//! Tests that validate MCP protocol compliance to prevent regressions.
//! These tests specifically target issues fixed in commits ffbe441 and a1af74c:
//! - Protocol version serialization format
//! - Tool inputSchema presence and validity
//! - JSON-RPC 2.0 response format
//!
//! Run with: `cargo test -p mcb-server --test unit mcp_protocol`

extern crate mcb_providers;

use std::net::TcpListener;
use std::sync::Arc;

use mcb_server::McpServer;
use mcb_server::transport::http::{HttpTransport, HttpTransportConfig};
use mcb_server::transport::types::{McpRequest, McpResponse};
use rocket::local::asynchronous::Client;
use rstest::*;
use tempfile::TempDir;

use crate::test_utils::test_fixtures::create_test_mcp_server;

/// Get a random available port
fn get_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port 0");
    let port = listener
        .local_addr()
        .expect("Failed to get local address")
        .port();
    drop(listener);
    port
}

struct TestContext {
    client: Client,
    _server: Arc<McpServer>,
    _temp: TempDir,
}

#[fixture]
async fn ctx() -> TestContext {
    let port = get_free_port();
    let (server_instance, temp) = create_test_mcp_server().await;
    let server = Arc::new(server_instance);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server.clone());

    let rocket = transport.rocket();
    let client = Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    TestContext {
        client,
        _server: server,
        _temp: temp,
    }
}

// =============================================================================
// PROTOCOL VERSION & INITIALIZE TESTS
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_initialize_response(#[future] ctx: TestContext) {
    let ctx = ctx.await;
    let request = McpRequest {
        method: "initialize".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = ctx
        .client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    assert!(mcp_response.error.is_none(), "Initialize should not error");

    let result = mcp_response.result.expect("Should have result");

    // Check protocol version (regression test for a1af74c)
    let version = result
        .get("protocolVersion")
        .expect("Should have protocolVersion");

    assert!(
        version.is_string(),
        "protocolVersion must be a JSON string. Got: {:?}",
        version
    );
    let version_str = version.as_str().unwrap();
    assert!(
        !version_str.contains("ProtocolVersion"),
        "protocolVersion has Debug format leak: {}",
        version_str
    );
    assert!(
        version_str.contains("-"),
        "protocolVersion should be date-formatted: {}",
        version_str
    );

    // Check serverInfo
    let server_info = result.get("serverInfo").expect("Should have serverInfo");
    assert!(server_info.is_object(), "serverInfo should be an object");
    let name = server_info.get("name").expect("Should have name");
    assert!(
        name.is_string() && !name.as_str().unwrap().is_empty(),
        "Invalid name"
    );
    assert!(
        server_info
            .get("version")
            .expect("Should have version")
            .is_string(),
        "Invalid version"
    );

    // Check capabilities
    let capabilities = result
        .get("capabilities")
        .expect("Should have capabilities");
    assert!(capabilities.is_object(), "capabilities should be an object");
    assert!(
        capabilities
            .get("tools")
            .expect("Should have tools")
            .is_object(),
        "tools cap should be object"
    );
}

// =============================================================================
// TOOL SCHEMA VALIDATION TESTS
// =============================================================================

#[rstest]
#[tokio::test]
async fn test_tools_schemas(#[future] ctx: TestContext) {
    let ctx = ctx.await;
    let request = McpRequest {
        method: "tools/list".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = ctx
        .client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");
    let result = mcp_response.result.expect("Should have result");

    let tools = result.get("tools").expect("Should have tools array");
    let tools_array = tools.as_array().expect("tools should be array");
    assert!(!tools_array.is_empty(), "Should have at least one tool");

    // Verify all tools have valid schemas
    for tool in tools_array {
        let tool_name = tool
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");
        let schema = tool.get("inputSchema").expect("Missing inputSchema");

        // Regression check: not null
        assert!(
            !schema.is_null(),
            "Tool '{}' has null inputSchema",
            tool_name
        );
        assert!(
            schema.is_object(),
            "Tool '{}' inputSchema should be object",
            tool_name
        );

        let schema_type = schema.get("type").expect("Missing type in schema");
        assert_eq!(
            schema_type.as_str(),
            Some("object"),
            "Schema type must be object"
        );

        assert!(
            schema
                .get("properties")
                .expect("Missing properties")
                .is_object(),
            "properties must be object"
        );
    }

    // Verify specific tools requirements
    let index_tool = tools_array
        .iter()
        .find(|t| t.get("name").and_then(|n| n.as_str()) == Some("index"))
        .expect("index tool missing");
    let required = index_tool
        .get("inputSchema")
        .unwrap()
        .get("required")
        .expect("req missing")
        .as_array()
        .expect("req array");
    assert!(
        required.iter().any(|v| v.as_str() == Some("action")),
        "index tool must require action"
    );

    let search_tool = tools_array
        .iter()
        .find(|t| t.get("name").and_then(|n| n.as_str()) == Some("search"))
        .expect("search tool missing");
    let required = search_tool
        .get("inputSchema")
        .unwrap()
        .get("required")
        .expect("req missing")
        .as_array()
        .expect("req array");
    assert!(
        required.iter().any(|v| v.as_str() == Some("query")),
        "search tool must require query"
    );
}

// =============================================================================
// JSON-RPC 2.0 FORMAT TESTS
// =============================================================================

#[rstest]
#[case("initialize")]
#[case("tools/list")]
#[case("ping")]
#[tokio::test]
async fn test_response_has_jsonrpc_field(#[future] ctx: TestContext, #[case] method: &str) {
    let ctx = ctx.await;
    let request = McpRequest {
        method: method.to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = ctx
        .client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    assert_eq!(
        mcp_response.jsonrpc, "2.0",
        "Response for '{}' should have jsonrpc: \"2.0\"",
        method
    );
}

#[rstest]
#[case(serde_json::json!(42))]
#[case(serde_json::json!("test-id-123"))]
#[tokio::test]
async fn test_response_echoes_request_id(
    #[future] ctx: TestContext,
    #[case] id: serde_json::Value,
) {
    let ctx = ctx.await;
    let request = McpRequest {
        method: "ping".to_string(),
        params: None,
        id: Some(id.clone()),
    };

    let response = ctx
        .client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    assert_eq!(mcp_response.id, Some(id));
}

#[rstest]
#[tokio::test]
async fn test_error_response_structure(#[future] ctx: TestContext) {
    let ctx = ctx.await;
    let request = McpRequest {
        method: "nonexistent/method".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = ctx
        .client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    assert!(mcp_response.error.is_some(), "Should have error");
    let error = mcp_response.error.unwrap();
    assert_eq!(error.code, -32601, "Error code -32601");
    assert!(!error.message.is_empty(), "Error message not empty");
}
