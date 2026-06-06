//! JSON field extraction helpers for freeform MCP data payloads.

use rmcp::model::CallToolResult;
use serde_json::{Map, Value};

use super::tool_error;

/// Requires a string value from a JSON object, returning an error if missing.
///
/// # Errors
/// Returns an error when the key is missing or value is not a string.
pub fn require_str(data: &Map<String, Value>, key: &str) -> Result<String, CallToolResult> {
    data.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| tool_error(format!("Missing required field: {key}")))
}

/// Requires an i64 value from a JSON object, returning an error if missing.
///
/// # Errors
/// Returns an error when the key is missing or value is not an integer.
pub fn require_i64(data: &Map<String, Value>, key: &str) -> Result<i64, CallToolResult> {
    data.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| tool_error(format!("Missing required field: {key}")))
}

/// Requires an i32 value from a JSON object, returning an error if missing or out of range.
///
/// # Errors
/// Returns an error when the key is missing, value is not an integer, or exceeds i32 range.
pub fn require_i32(data: &Map<String, Value>, key: &str) -> Result<i32, CallToolResult> {
    data.get(key)
        .and_then(Value::as_i64)
        .and_then(|value| value.try_into().ok())
        .ok_or_else(|| tool_error(format!("Missing required field: {key}")))
}

/// Requires a boolean value from a JSON object, returning an error if missing.
///
/// # Errors
/// Returns an error when the key is missing or value is not a boolean.
pub fn require_bool(data: &Map<String, Value>, key: &str) -> Result<bool, CallToolResult> {
    data.get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| tool_error(format!("Missing required field: {key}")))
}

/// Extracts an optional string value from a JSON object.
pub fn opt_str(data: &Map<String, Value>, key: &str) -> Option<String> {
    data.get(key).and_then(Value::as_str).map(str::to_owned)
}

/// Extracts an optional boolean value from a JSON object.
pub fn opt_bool(data: &Map<String, Value>, key: &str) -> Option<bool> {
    data.get(key).and_then(Value::as_bool)
}

/// Extracts a string array from a JSON object, defaulting to empty if missing.
pub fn str_vec(data: &Map<String, Value>, key: &str) -> Vec<String> {
    data.get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}
