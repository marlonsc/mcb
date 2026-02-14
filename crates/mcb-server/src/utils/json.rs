//! JSON utilities for MCP server tool handlers.
//!
use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};

/// Extension methods for typed extraction from JSON object maps.
pub trait JsonMapExt {
    /// Returns the string value for `key`, if present and of string type.
    fn string(&self, key: &str) -> Option<String>;

    /// Returns the required string value for `key`, or an MCP-compatible error result.
    fn required_string(&self, key: &str) -> Result<String, CallToolResult>;

    /// Returns the i64 value for `key`, if present and numeric.
    fn int64(&self, key: &str) -> Option<i64>;

    /// Returns the i32 value for `key`, if present and convertible from i64.
    fn int32(&self, key: &str) -> Option<i32>;

    /// Returns the f32 value for `key`, if present and numeric.
    fn float32(&self, key: &str) -> Option<f32>;

    /// Returns the bool value for `key`, if present and boolean.
    fn boolean(&self, key: &str) -> Option<bool>;

    /// Returns a string vector from array field `key`, filtering non-string items.
    fn string_list(&self, key: &str) -> Vec<String>;
}

impl JsonMapExt for Map<String, Value> {
    fn string(&self, key: &str) -> Option<String> {
        self.get(key)
            .and_then(|value| value.as_str())
            .map(str::to_string)
    }

    fn required_string(&self, key: &str) -> Result<String, CallToolResult> {
        self.string(key).ok_or_else(|| {
            CallToolResult::error(vec![Content::text(format!(
                "Missing required field: {key}"
            ))])
        })
    }

    fn int64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|value| value.as_i64())
    }

    fn int32(&self, key: &str) -> Option<i32> {
        self.get(key)
            .and_then(|value| value.as_i64())
            .and_then(|v| v.try_into().ok())
    }

    fn float32(&self, key: &str) -> Option<f32> {
        self.get(key)
            .and_then(|value| value.as_f64())
            .map(|v| v as f32)
    }

    fn boolean(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|value| value.as_bool())
    }

    fn string_list(&self, key: &str) -> Vec<String> {
        self.get(key)
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Extracts a JSON object map from an optional JSON value.
///
/// Returns a reference to the underlying map if the value is an object, or `None` otherwise.
pub fn json_map(data: &Option<Value>) -> Option<&Map<String, Value>> {
    data.as_ref().and_then(|value| value.as_object())
}
