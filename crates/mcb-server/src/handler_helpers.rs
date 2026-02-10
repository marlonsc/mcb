use rmcp::model::{CallToolResult, Content, ErrorData as McpError};
use serde::Serialize;

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
