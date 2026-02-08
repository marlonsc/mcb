//! Tests for transport layer types

use mcb_server::transport::types::{McpRequest, McpResponse};

#[test]
fn test_mcp_request_serialization() {
    let request = McpRequest {
        method: "tools/list".to_string(),
        params: None,
        id: Some(serde_json::json!(1)),
    };
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("tools/list"));
}

#[test]
fn test_mcp_request_with_params() {
    let request = McpRequest {
        method: "tools/call".to_string(),
        params: Some(serde_json::json!({"name": "search"})),
        id: Some(serde_json::json!(2)),
    };
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("tools/call"));
    assert!(json.contains("search"));
}

#[test]
fn test_mcp_response_success() {
    let response = McpResponse::success(
        Some(serde_json::json!(1)),
        serde_json::json!({"result": "ok"}),
    );
    assert!(response.result.is_some());
    assert!(response.error.is_none());
    assert_eq!(response.jsonrpc, "2.0");
}

#[test]
fn test_mcp_response_error() {
    let response = McpResponse::error(Some(serde_json::json!(1)), -32600, "Invalid request");
    assert!(response.result.is_none());
    assert!(response.error.is_some());
    let err = response.error.unwrap();
    assert_eq!(err.code, -32600);
    assert_eq!(err.message, "Invalid request");
}

#[test]
fn test_mcp_response_serialization_roundtrip() {
    let response =
        McpResponse::success(Some(serde_json::json!(1)), serde_json::json!({"tools": []}));
    let json = serde_json::to_string(&response).unwrap();
    let deserialized: McpResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.jsonrpc, "2.0");
    assert!(deserialized.result.is_some());
}

#[test]
fn test_mcp_request_deserialization() {
    let json = r#"{"method":"ping","params":null,"id":1}"#;
    let request: McpRequest = serde_json::from_str(json).unwrap();
    assert_eq!(request.method, "ping");
    assert!(request.params.is_none());
}
