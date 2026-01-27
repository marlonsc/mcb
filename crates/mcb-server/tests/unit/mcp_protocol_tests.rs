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

use mcb_server::transport::http::{HttpTransport, HttpTransportConfig};
use mcb_server::transport::types::{McpRequest, McpResponse};
use std::net::TcpListener;
use std::sync::Arc;

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

// =============================================================================
// PROTOCOL VERSION TESTS - Prevent regression of commit a1af74c
// =============================================================================

/// Test that initialize response returns protocolVersion as a string, not Debug format
///
/// This prevents regression of the fix in commit a1af74c where protocolVersion
/// was incorrectly returning `ProtocolVersion("2025-03-26")` instead of `"2025-03-26"`.
#[tokio::test]
async fn test_initialize_response_protocol_version_is_string() {
    let port = get_free_port();
    let server = Arc::new(create_test_mcp_server().await);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    let request = McpRequest {
        method: "initialize".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = client
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
    let version = result
        .get("protocolVersion")
        .expect("Should have protocolVersion");

    // CRITICAL: Protocol version MUST be a string, not Debug format
    assert!(
        version.is_string(),
        "protocolVersion must be a JSON string, not Debug format. Got: {:?}",
        version
    );

    let version_str = version.as_str().unwrap();

    // Should NOT contain "ProtocolVersion(" which indicates Debug format leak
    assert!(
        !version_str.contains("ProtocolVersion"),
        "protocolVersion has Debug format leak: {}",
        version_str
    );

    // Should be a valid version string (e.g., "2025-03-26")
    assert!(
        version_str.contains("-"),
        "protocolVersion should be date-formatted version string: {}",
        version_str
    );
}

/// Test that initialize response has required serverInfo structure
#[tokio::test]
async fn test_initialize_response_has_server_info() {
    let port = get_free_port();
    let server = Arc::new(create_test_mcp_server().await);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    let request = McpRequest {
        method: "initialize".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");
    let result = mcp_response.result.expect("Should have result");

    // Verify serverInfo structure
    let server_info = result.get("serverInfo").expect("Should have serverInfo");
    assert!(server_info.is_object(), "serverInfo should be an object");

    let name = server_info
        .get("name")
        .expect("serverInfo should have name");
    assert!(name.is_string(), "serverInfo.name should be a string");
    assert!(
        !name.as_str().unwrap().is_empty(),
        "serverInfo.name should not be empty"
    );

    let version = server_info
        .get("version")
        .expect("serverInfo should have version");
    assert!(version.is_string(), "serverInfo.version should be a string");
}

/// Test that initialize response has capabilities structure
#[tokio::test]
async fn test_initialize_response_has_capabilities() {
    let port = get_free_port();
    let server = Arc::new(create_test_mcp_server().await);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    let request = McpRequest {
        method: "initialize".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");
    let result = mcp_response.result.expect("Should have result");

    // Verify capabilities structure
    let capabilities = result
        .get("capabilities")
        .expect("Should have capabilities");
    assert!(capabilities.is_object(), "capabilities should be an object");

    // Verify tools capability is present
    let tools = capabilities
        .get("tools")
        .expect("capabilities should have tools");
    assert!(tools.is_object(), "capabilities.tools should be an object");
}

// =============================================================================
// TOOL SCHEMA VALIDATION TESTS - Prevent regression of commit a1af74c
// =============================================================================

/// Test that all tools have non-null inputSchema
///
/// This prevents regression of the fix in commit a1af74c where inputSchema
/// was incorrectly returning null instead of the actual JSON Schema.
#[tokio::test]
async fn test_tools_list_has_input_schemas() {
    let port = get_free_port();
    let server = Arc::new(create_test_mcp_server().await);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    let request = McpRequest {
        method: "tools/list".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = client
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

    // CRITICAL: Every tool MUST have a non-null inputSchema
    for tool in tools_array {
        let tool_name = tool
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");
        let schema = tool.get("inputSchema");

        assert!(
            schema.is_some(),
            "Tool '{}' is missing inputSchema field",
            tool_name
        );

        let schema_value = schema.unwrap();
        assert!(
            !schema_value.is_null(),
            "Tool '{}' has null inputSchema - regression of commit a1af74c fix!",
            tool_name
        );

        // Schema should be a valid JSON Schema object
        assert!(
            schema_value.is_object(),
            "Tool '{}' inputSchema should be an object, got: {:?}",
            tool_name,
            schema_value
        );
    }
}

/// Test that index_codebase tool schema has required 'path' field
#[tokio::test]
async fn test_index_codebase_schema_has_required_path() {
    let port = get_free_port();
    let server = Arc::new(create_test_mcp_server().await);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    let request = McpRequest {
        method: "tools/list".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");
    let result = mcp_response.result.expect("Should have result");

    let tools = result.get("tools").unwrap().as_array().unwrap();
    let index_tool = tools
        .iter()
        .find(|t| t.get("name").and_then(|n| n.as_str()) == Some("index_codebase"))
        .expect("Should have index_codebase tool");

    let schema = index_tool.get("inputSchema").unwrap();

    // Check for required field
    let required = schema.get("required");
    assert!(
        required.is_some(),
        "index_codebase schema should have 'required' field"
    );

    let required_array = required
        .unwrap()
        .as_array()
        .expect("required should be array");
    assert!(
        required_array.iter().any(|v| v.as_str() == Some("path")),
        "index_codebase schema should require 'path' field. Required: {:?}",
        required_array
    );
}

/// Test that search_code tool schema has required 'query' field
#[tokio::test]
async fn test_search_code_schema_has_required_query() {
    let port = get_free_port();
    let server = Arc::new(create_test_mcp_server().await);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    let request = McpRequest {
        method: "tools/list".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");
    let result = mcp_response.result.expect("Should have result");

    let tools = result.get("tools").unwrap().as_array().unwrap();
    let search_tool = tools
        .iter()
        .find(|t| t.get("name").and_then(|n| n.as_str()) == Some("search_code"))
        .expect("Should have search_code tool");

    let schema = search_tool.get("inputSchema").unwrap();

    // Check for required field
    let required = schema.get("required");
    assert!(
        required.is_some(),
        "search_code schema should have 'required' field"
    );

    let required_array = required
        .unwrap()
        .as_array()
        .expect("required should be array");
    assert!(
        required_array.iter().any(|v| v.as_str() == Some("query")),
        "search_code schema should require 'query' field. Required: {:?}",
        required_array
    );
}

/// Test that all tool schemas have valid JSON Schema structure
#[tokio::test]
async fn test_tool_schemas_have_valid_structure() {
    let port = get_free_port();
    let server = Arc::new(create_test_mcp_server().await);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    let request = McpRequest {
        method: "tools/list".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");
    let result = mcp_response.result.expect("Should have result");

    let tools = result.get("tools").unwrap().as_array().unwrap();

    for tool in tools {
        let tool_name = tool
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");
        let schema = tool.get("inputSchema").unwrap();

        // Verify JSON Schema structure
        let schema_type = schema.get("type");
        assert!(
            schema_type.is_some(),
            "Tool '{}' schema should have 'type' field",
            tool_name
        );
        assert_eq!(
            schema_type.unwrap().as_str(),
            Some("object"),
            "Tool '{}' schema type should be 'object'",
            tool_name
        );

        // Properties should exist (even if empty)
        let properties = schema.get("properties");
        assert!(
            properties.is_some(),
            "Tool '{}' schema should have 'properties' field",
            tool_name
        );
        assert!(
            properties.unwrap().is_object(),
            "Tool '{}' properties should be an object",
            tool_name
        );
    }
}

// =============================================================================
// JSON-RPC 2.0 FORMAT TESTS
// =============================================================================

/// Test that all responses have jsonrpc field set to "2.0"
#[tokio::test]
async fn test_response_has_jsonrpc_field() {
    let port = get_free_port();
    let server = Arc::new(create_test_mcp_server().await);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    // Test multiple methods
    let methods = vec!["initialize", "tools/list", "ping"];

    for method in methods {
        let request = McpRequest {
            method: method.to_string(),
            params: None,
            id: Some(serde_json::json!(1)),
        };

        let response = client
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
}

/// Test that response echoes back the request id
#[tokio::test]
async fn test_response_echoes_request_id() {
    let port = get_free_port();
    let server = Arc::new(create_test_mcp_server().await);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    // Test with numeric id
    let request = McpRequest {
        method: "ping".to_string(),
        params: None,
        id: Some(serde_json::json!(42)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    assert_eq!(
        mcp_response.id,
        Some(serde_json::json!(42)),
        "Response should echo numeric request id"
    );

    // Test with string id
    let request = McpRequest {
        method: "ping".to_string(),
        params: None,
        id: Some(serde_json::json!("test-id-123")),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    assert_eq!(
        mcp_response.id,
        Some(serde_json::json!("test-id-123")),
        "Response should echo string request id"
    );
}

/// Test that error responses have code and message fields
#[tokio::test]
async fn test_error_response_has_code_and_message() {
    let port = get_free_port();
    let server = Arc::new(create_test_mcp_server().await);

    let http_config = HttpTransportConfig::localhost(port);
    let transport = HttpTransport::new(http_config, server);

    let rocket = transport.rocket();
    let client = rocket::local::asynchronous::Client::tracked(rocket)
        .await
        .expect("Failed to create test client");

    // Send unknown method to trigger error
    let request = McpRequest {
        method: "nonexistent/method".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };

    let response = client
        .post("/mcp")
        .header(rocket::http::ContentType::JSON)
        .body(serde_json::to_string(&request).unwrap())
        .dispatch()
        .await;

    let body = response.into_string().await.expect("Response body");
    let mcp_response: McpResponse = serde_json::from_str(&body).expect("Parse response");

    assert!(
        mcp_response.error.is_some(),
        "Should have error for unknown method"
    );

    let error = mcp_response.error.unwrap();
    assert_eq!(
        error.code, -32601,
        "Error code should be -32601 (method not found)"
    );
    assert!(
        !error.message.is_empty(),
        "Error message should not be empty"
    );
}
