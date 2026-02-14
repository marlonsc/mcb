use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};

pub(crate) fn optional_data_map(data: &Option<Value>) -> Option<&Map<String, Value>> {
    data.as_ref().and_then(Value::as_object)
}

pub(crate) fn require_data_map<'a>(
    data: &'a Option<Value>,
    missing_message: &'static str,
) -> Result<&'a Map<String, Value>, CallToolResult> {
    optional_data_map(data)
        .ok_or_else(|| CallToolResult::error(vec![Content::text(missing_message)]))
}

pub(crate) fn require_str(data: &Map<String, Value>, key: &str) -> Result<String, CallToolResult> {
    data.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            CallToolResult::error(vec![Content::text(format!(
                "Missing required field: {key}"
            ))])
        })
}

pub(crate) fn opt_str(data: &Map<String, Value>, key: &str) -> Option<String> {
    data.get(key).and_then(Value::as_str).map(str::to_owned)
}

pub(crate) fn opt_bool(data: &Map<String, Value>, key: &str) -> Option<bool> {
    data.get(key).and_then(Value::as_bool)
}

pub(crate) fn str_vec(data: &Map<String, Value>, key: &str) -> Vec<String> {
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
