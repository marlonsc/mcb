use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};

pub(super) fn require_data_map<'a>(
    data: &'a Option<Value>,
    missing_message: &'static str,
) -> Result<&'a Map<String, Value>, CallToolResult> {
    data.as_ref()
        .and_then(Value::as_object)
        .ok_or_else(|| CallToolResult::error(vec![Content::text(missing_message)]))
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

pub(super) fn opt_bool(data: &Map<String, Value>, key: &str) -> Option<bool> {
    data.get(key).and_then(Value::as_bool)
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
