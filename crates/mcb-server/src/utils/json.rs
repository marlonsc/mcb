//! JSON utilities for MCP server tool handlers.
//!
use serde_json::{Map, Value};

/// Extracts a JSON object map from an optional JSON value.
///
/// Returns a reference to the underlying map if the value is an object, or `None` otherwise.
pub fn json_map(data: &Option<Value>) -> Option<&Map<String, Value>> {
    data.as_ref().and_then(|value| value.as_object())
}
