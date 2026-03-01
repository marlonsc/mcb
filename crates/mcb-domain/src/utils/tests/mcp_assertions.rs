//! MCP tool result invariant assertions.
//!
//! Centralized in `mcb-domain` so all test crates that verify MCP responses
//! share the same helpers. These depend only on `rmcp` model types.

use rmcp::model::CallToolResult;

/// Extract the first text element from a `CallToolResult`.
#[must_use]
pub fn error_text(result: &CallToolResult) -> String {
    serde_json::to_value(&result.content)
        .ok()
        .and_then(|value| value.as_array().cloned())
        .and_then(|items| items.first().cloned())
        .and_then(|item| item.get("text").cloned())
        .and_then(|text| text.as_str().map(ToOwned::to_owned))
        .unwrap_or_default()
}

/// Check if a tool result indicates an error.
#[must_use]
pub fn is_error(result: &CallToolResult) -> bool {
    result.is_error.unwrap_or(false)
}

/// Extract all text content blocks from a tool result, joined by newlines.
#[must_use]
pub fn extract_text(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Assert that a `CallToolResult` has `is_error=true` and contains
/// the expected message substring.
///
/// # Panics
/// Panics if result is not an error or message doesn't match.
pub fn assert_error_shape(result: &CallToolResult, expected_message: &str) {
    assert_eq!(result.is_error, Some(true), "expected is_error=true");

    let content_json_result = serde_json::to_value(&result.content);
    assert!(content_json_result.is_ok(), "serialize content");
    let content_json = match content_json_result {
        Ok(value) => value,
        Err(_) => return,
    };
    assert!(content_json.is_array(), "error content must be an array");
    assert!(
        content_json
            .as_array()
            .is_some_and(|items| items.first().and_then(|item| item.get("text")).is_some()),
        "error content must contain a text field"
    );

    let text = error_text(result);
    assert!(
        text.contains(expected_message),
        "expected '{expected_message}' in '{text}'"
    );
}

/// Assert the JSON-RPC error code is `-32602` (invalid params) and message contains substring.
///
/// # Panics
/// Panics if error code or message don't match.
pub fn assert_invalid_params(error: &rmcp::ErrorData, expected_substring: &str) {
    assert_eq!(
        error.code.0, -32602,
        "expected error code -32602, got {}",
        error.code.0
    );
    assert!(
        error.message.contains(expected_substring),
        "expected '{}' in error message: {}",
        expected_substring,
        error.message
    );
}

/// Assert that a tool call results in an error (either MCP-level or application-level).
///
/// The server may return errors as:
/// - `Err(McpError)` — JSON-RPC level (e.g., invalid params, unknown variant)
/// - `Ok(CallToolResult { is_error: true })` — application-level error
///
/// If `expected_keywords` is non-empty, at least one keyword must appear in the error.
///
/// # Panics
/// Panics if result is not an error or expected keywords are not found.
pub fn assert_tool_error(
    result: Result<CallToolResult, Box<dyn std::error::Error>>,
    expected_keywords: &[&str],
) {
    match result {
        Err(e) => {
            if !expected_keywords.is_empty() {
                let msg = e.to_string().to_lowercase();
                assert!(
                    expected_keywords
                        .iter()
                        .any(|k| msg.contains(&k.to_lowercase())),
                    "Expected error containing one of {expected_keywords:?}, got: {e}"
                );
            }
        }
        Ok(r) if is_error(&r) => {
            if !expected_keywords.is_empty() {
                let text = extract_text(&r).to_lowercase();
                assert!(
                    expected_keywords
                        .iter()
                        .any(|k| text.contains(&k.to_lowercase())),
                    "Expected error containing one of {expected_keywords:?}, got: {}",
                    extract_text(&r)
                );
            }
        }
        Ok(r) => {
            unreachable!("Expected error, got success: {}", extract_text(&r));
        }
    }
}
