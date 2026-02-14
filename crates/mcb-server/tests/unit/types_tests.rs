//! Tests for transport layer types

use mcb_server::transport::types::{McpRequest, McpResponse};
use rstest::rstest;

#[rstest]
#[case("tools/list", None, 1, "tools/list", None)]
#[case(
    "tools/call",
    Some(serde_json::json!({"name": "search"})),
    2,
    "tools/call",
    Some("search")
)]
#[test]
fn test_mcp_request_serialization(
    #[case] method: &str,
    #[case] params: Option<serde_json::Value>,
    #[case] id: i64,
    #[case] expected_method: &str,
    #[case] expected_param_fragment: Option<&str>,
) {
    let request = McpRequest {
        method: method.to_string(),
        params,
        id: Some(serde_json::json!(id)),
    };
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains(expected_method));
    if let Some(fragment) = expected_param_fragment {
        assert!(json.contains(fragment));
    }
}

#[rstest]
#[case(true, Some(serde_json::json!(1)), -32600, "Invalid request")]
#[case(false, Some(serde_json::json!(1)), -32600, "Invalid request")]
#[test]
fn test_mcp_response_shapes(
    #[case] is_error: bool,
    #[case] id: Option<serde_json::Value>,
    #[case] error_code: i32,
    #[case] error_message: &str,
) {
    let response = if is_error {
        McpResponse::error(id, error_code, error_message)
    } else {
        McpResponse::success(id, serde_json::json!({"result": "ok"}))
    };

    assert_eq!(response.jsonrpc, "2.0");
    if is_error {
        assert!(response.result.is_none());
        let err = response.error.expect("expected error payload");
        assert_eq!(err.code, error_code);
        assert_eq!(err.message, error_message);
    } else {
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }
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
