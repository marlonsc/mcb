//! MCP HTTP test helpers — centralized in `mcb_domain`.
//!
//! Provides request builders and assertion helpers for MCP protocol testing.
//! All crates MUST import these from `mcb_domain::test_http_mcp` (or
//! `mcb_domain::utils::tests::http_mcp`) instead of defining their own.
//!
//! **Documentation**: [docs/modules/domain.md#testing-utilities](../../../../docs/modules/domain.md#testing-utilities)

use crate::protocol::{McpRequest, McpResponse};
use crate::utils::tests::utils::TestResult;

// ---------------------------------------------------------------------------
// Header helpers
// ---------------------------------------------------------------------------

/// Extract header value by name (case-insensitive key match).
#[must_use]
pub fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    let lower = name.to_lowercase();
    headers
        .iter()
        .find(|(k, _)| k.to_lowercase() == lower)
        .map(|(_, v)| v.as_str())
}

// ---------------------------------------------------------------------------
// Request builders
// ---------------------------------------------------------------------------

/// Build a `tools/list` MCP request.
#[must_use]
pub fn tools_list_request() -> McpRequest {
    McpRequest {
        method: "tools/list".to_owned(),
        params: None,
        id: Some(serde_json::json!(1)),
    }
}

/// Build a `tools/call` MCP request for the given tool.
#[must_use]
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

/// Build a `tools/call` request with custom arguments.
#[must_use]
pub fn tools_call_request_with_args(tool_name: &str, arguments: &serde_json::Value) -> McpRequest {
    McpRequest {
        method: "tools/call".to_owned(),
        params: Some(serde_json::json!({
            "name": tool_name,
            "arguments": arguments
        })),
        id: Some(serde_json::json!(1)),
    }
}

/// Build an `initialize` MCP request.
#[must_use]
#[allow(dead_code)]
pub fn initialize_request() -> McpRequest {
    McpRequest {
        method: "initialize".to_owned(),
        params: None,
        id: Some(serde_json::json!(1)),
    }
}

// ---------------------------------------------------------------------------
// Response assertions
// ---------------------------------------------------------------------------

/// Assert that the response is a success (no error, has result).
///
/// # Errors
///
/// Returns an error if the response has an error or no result.
pub fn assert_mcp_success(response: &McpResponse) -> TestResult<&serde_json::Value> {
    if let Some(ref err) = response.error {
        return Err(format!(
            "Expected success, got error: {} ({})",
            err.message, err.code
        )
        .into());
    }
    response
        .result
        .as_ref()
        .ok_or_else(|| "Expected result in success response, got None".into())
}

/// Assert that the response is an error with the expected code.
///
/// # Errors
///
/// Returns an error if the response is not an error or has a different code.
pub fn assert_mcp_error(response: &McpResponse, expected_code: i32) -> TestResult<&McpResponse> {
    let err = response
        .error
        .as_ref()
        .ok_or("Expected error response, got success")?;
    if err.code != expected_code {
        return Err(format!(
            "Expected error code {expected_code}, got {} ({})",
            err.code, err.message
        )
        .into());
    }
    Ok(response)
}

/// Build an `initialize` success response (for mock/stub usage).
#[must_use]
pub fn initialize_response(id: Option<serde_json::Value>) -> McpResponse {
    McpResponse::from_success(
        id,
        serde_json::json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": "mcb",
                "version": "test"
            },
            "capabilities": {
                "tools": {}
            }
        }),
    )
}
