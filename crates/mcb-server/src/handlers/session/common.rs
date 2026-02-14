use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};

use crate::args::SessionArgs;
use mcb_domain::value_objects::ids::SessionId;

pub(super) fn require_session_id(args: &SessionArgs) -> Result<&SessionId, CallToolResult> {
    args.session_id
        .as_ref()
        .ok_or_else(|| CallToolResult::error(vec![Content::text("Missing session_id")]))
}

pub(super) fn require_session_id_str(args: &SessionArgs) -> Result<&str, CallToolResult> {
    require_session_id(args).map(|id| id.as_str())
}

pub(super) fn require_data_map<'a>(
    data: &'a Option<Value>,
    missing_message: &'static str,
) -> Result<&'a Map<String, Value>, CallToolResult> {
    data.as_ref()
        .and_then(Value::as_object)
        .ok_or_else(|| CallToolResult::error(vec![Content::text(missing_message)]))
}

pub(super) fn optional_data_map(data: &Option<Value>) -> Option<&Map<String, Value>> {
    data.as_ref().and_then(Value::as_object)
}

pub(super) fn require_str(data: &Map<String, Value>, key: &str) -> Result<String, CallToolResult> {
    data.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            CallToolResult::error(vec![Content::text(format!(
                "Missing required field: {key}"
            ))])
        })
}

pub(super) fn opt_str(data: &Map<String, Value>, key: &str) -> Option<String> {
    data.get(key).and_then(Value::as_str).map(str::to_owned)
}

pub(super) fn str_vec(data: &Map<String, Value>, key: &str) -> Vec<String> {
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

pub(super) fn parse_agent_type(
    value: &str,
) -> Result<mcb_domain::entities::agent::AgentType, McpError> {
    value
        .parse::<mcb_domain::entities::agent::AgentType>()
        .map_err(|e: String| McpError::invalid_params(e, None))
}
