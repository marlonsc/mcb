use std::time::{SystemTime, UNIX_EPOCH};

use mcb_domain::error::Error;
use mcb_domain::value_objects::OrgContext;
use rmcp::model::{CallToolResult, Content, ErrorData as McpError};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Returns the current Unix timestamp in seconds.
pub fn current_timestamp() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => i64::try_from(duration.as_secs()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

/// Returns the required `id` parameter or an MCP invalid params error.
pub fn require_id(id: &Option<String>) -> Result<String, McpError> {
    id.clone()
        .ok_or_else(|| McpError::invalid_params("id required", None))
}

/// Serializes a value into pretty JSON and wraps it in a successful MCP tool result.
pub fn ok_json<T: Serialize>(val: &T) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(val)
        .map_err(|_| McpError::internal_error("serialization failed", None))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Wraps plain text in a successful MCP tool result.
pub fn ok_text(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

/// Resolves the organization id, preferring explicit input over the current context default.
pub fn resolve_org_id(explicit: Option<&str>) -> String {
    if let Some(org_id) = explicit {
        return org_id.to_string();
    }
    OrgContext::current().id_str().to_string()
}

/// Normalizes optional identifier input by trimming whitespace and discarding empty values.
pub fn normalize_identifier(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

/// Resolves identifier precedence between explicit args and payload fields.
///
/// Returns an error when both values are present but conflict.
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

/// Deserializes required request data into the target type.
pub fn require_data<T: DeserializeOwned>(
    data: Option<serde_json::Value>,
    msg: &'static str,
) -> Result<T, McpError> {
    let value = data.ok_or_else(|| McpError::invalid_params(msg, None))?;
    serde_json::from_value(value).map_err(|_| McpError::invalid_params("invalid data", None))
}

/// Maps domain errors to opaque MCP-safe errors.
pub fn map_opaque_error<T>(result: Result<T, Error>) -> Result<T, McpError> {
    result.map_err(crate::error_mapping::to_opaque_mcp_error)
}
