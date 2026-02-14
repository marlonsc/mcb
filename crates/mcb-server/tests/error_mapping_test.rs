//! Tests for MCP/domain error mapping helpers.

use mcb_domain::error::Error;
use mcb_server::error_mapping::{to_contextual_tool_error, to_opaque_mcp_error};
use rstest::rstest;

#[rstest]
#[case(Error::NotFound { resource: "test".to_string() }, "Not found: test")]
#[case(Error::Internal { message: "secret".to_string() }, "internal server error")]
fn test_to_opaque_mcp_error(#[case] err: Error, #[case] expected_message: &str) {
    let mcp_err = to_opaque_mcp_error(err);
    assert_eq!(mcp_err.message, expected_message);
}

#[rstest]
#[case(Error::NotFound { resource: "item".to_string() }, "Not found: item")]
#[case(
    Error::Database {
        message: "db fail".to_string(),
        source: None,
    },
    "Database error: db fail"
)]
#[test]
fn test_to_contextual_tool_error(#[case] err: Error, #[case] expected: &str) {
    let result = to_contextual_tool_error(err);
    assert!(result.is_error.unwrap_or(false));
    let content_json = serde_json::to_value(&result.content[0]).expect("serialize content");
    let text = content_json
        .get("text")
        .and_then(|value| value.as_str())
        .expect("text content");
    assert_eq!(text, expected);
}
