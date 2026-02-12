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

pub fn require_id(id: &Option<String>) -> Result<String, McpError> {
    id.clone()
        .ok_or_else(|| McpError::invalid_params("id required", None))
}

pub fn ok_json<T: Serialize>(val: &T) -> Result<CallToolResult, McpError> {
    let json = serde_json::to_string_pretty(val)
        .map_err(|_| McpError::internal_error("serialization failed", None))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub fn ok_text(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(msg)]))
}

pub fn resolve_org_id(explicit: Option<&str>) -> String {
    if let Some(org_id) = explicit {
        return org_id.to_string();
    }
    OrgContext::current().id_str().to_string()
}

pub fn require_data<T: DeserializeOwned>(
    data: Option<serde_json::Value>,
    msg: &'static str,
) -> Result<T, McpError> {
    let value = data.ok_or_else(|| McpError::invalid_params(msg, None))?;
    serde_json::from_value(value).map_err(|_| McpError::invalid_params("invalid data", None))
}

pub fn map_opaque_error<T>(result: Result<T, Error>) -> Result<T, McpError> {
    result.map_err(crate::error_mapping::to_opaque_mcp_error)
}
