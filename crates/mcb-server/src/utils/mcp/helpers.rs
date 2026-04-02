//! Core MCP response helpers and identifier resolution.

use mcb_domain::error::Error;
use mcb_domain::value_objects::OrgContext;
use rmcp::model::{CallToolResult, Content, ErrorData as McpError};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error_mapping::safe_internal_error;
use crate::utils::json::json_map;

/// Returns the required `id` parameter or an MCP invalid params error.
///
/// # Errors
/// Returns an error when `id` is missing.
pub fn require_id(id: &Option<String>) -> Result<String, McpError> {
    id.clone()
        .ok_or_else(|| McpError::invalid_params("id required", None))
}

/// Serializes a value into pretty JSON and wraps it in a successful MCP tool result.
///
/// # Errors
/// Returns an error when JSON serialization fails.
pub fn ok_json<T: Serialize>(val: &T) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(val)
        .map_err(|e| safe_internal_error("json serialization", &e))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Wraps plain text in a successful MCP tool result.
///
/// # Errors
/// Returns an error when MCP content encoding fails.
pub fn ok_text(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Builds a tool error result with a contextual message.
pub fn tool_error(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(msg)])
}

/// Resolves the organization id, preferring explicit input over the current context default.
#[must_use]
pub fn resolve_org_id(explicit: Option<&str>) -> String {
    if let Some(org_id) = explicit {
        return org_id.to_owned();
    }
    OrgContext::default().id_str().clone()
}

/// Normalizes optional identifier input by trimming whitespace and discarding empty values.
#[must_use]
pub fn normalize_identifier(value: Option<&str>) -> Option<String> {
    let raw = value?;

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

/// Resolves identifier precedence between explicit args and payload fields.
///
/// Returns an error when both values are present but conflict.
///
/// # Errors
/// Returns an error when argument and payload values conflict.
pub fn resolve_identifier_precedence(
    field: &str,
    args_value: Option<&str>,
    payload_value: Option<&str>,
) -> Result<Option<String>, McpError> {
    let args_normalized = normalize_identifier(args_value);
    let payload_normalized = normalize_identifier(payload_value);

    if let (Some(arg), Some(payload)) = (&args_normalized, &payload_normalized)
        && arg != payload
    {
        return Err(McpError::invalid_params(
            format!("conflicting {field} between args and data"),
            None,
        ));
    }

    Ok(args_normalized.or(payload_normalized))
}

/// Resolve and require an identifier from args/payload precedence.
///
/// Returns an invalid-params error with `required_message` when both values are absent.
///
/// # Errors
/// Returns an error when values conflict or no identifier is available.
pub fn require_resolved_identifier(
    field: &str,
    args_value: Option<&str>,
    payload_value: Option<&str>,
    required_message: &'static str,
) -> Result<String, McpError> {
    resolve_identifier_precedence(field, args_value, payload_value)?
        .ok_or_else(|| McpError::invalid_params(required_message, None))
}

/// Deserializes required request data into the target type.
///
/// # Errors
/// Returns an error when data is missing or deserialization fails.
pub fn require_data<T: DeserializeOwned>(
    data: Option<serde_json::Value>,
    msg: &'static str,
) -> Result<T, McpError> {
    let value = data.ok_or_else(|| McpError::invalid_params(msg, None))?;
    serde_json::from_value(value).map_err(|_| McpError::invalid_params("invalid data", None))
}

/// Maps domain errors to opaque MCP-safe errors.
///
/// # Errors
/// Returns an error when the domain result is an error.
pub fn map_opaque_error<T>(result: Result<T, Error>) -> Result<T, McpError> {
    result.map_err(|e| crate::error_mapping::to_opaque_mcp_error(&e))
}

/// Requires a JSON object from an optional Value, returning an error if missing.
///
/// # Errors
/// Returns an error when the payload is missing or not an object.
pub fn require_data_map<'a>(
    data: &'a Option<serde_json::Value>,
    missing_message: &'static str,
) -> Result<&'a serde_json::Map<String, serde_json::Value>, CallToolResult> {
    json_map(data).ok_or_else(|| tool_error(missing_message))
}
