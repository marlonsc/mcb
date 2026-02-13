//! JSON utilities for MCP server tool handlers.
//!
//! Provides common functions for extracting typed values from JSON objects.

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
        get_str(self, key)
    }

    fn required_string(&self, key: &str) -> Result<String, CallToolResult> {
        get_required_str(self, key)
    }

    fn int64(&self, key: &str) -> Option<i64> {
        get_i64(self, key)
    }

    fn int32(&self, key: &str) -> Option<i32> {
        get_i32(self, key)
    }

    fn float32(&self, key: &str) -> Option<f32> {
        get_f32(self, key)
    }

    fn boolean(&self, key: &str) -> Option<bool> {
        get_bool(self, key)
    }

    fn string_list(&self, key: &str) -> Vec<String> {
        get_string_list(self, key)
    }
}

/// Extracts a JSON object map from an optional JSON value.
///
/// Returns a reference to the underlying map if the value is an object, or `None` otherwise.
pub fn json_map(data: &Option<Value>) -> Option<&Map<String, Value>> {
    data.as_ref().and_then(|value| value.as_object())
}

/// Extracts a string value from a JSON object by key.
///
/// Returns the string value if the key exists and contains a string, or `None` otherwise.
pub fn get_str(data: &Map<String, Value>, key: &str) -> Option<String> {
    data.get(key)
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

/// Extracts a required string value from a JSON object by key.
///
/// Returns the string value if the key exists and contains a string.
/// Returns an error result if the key is missing or does not contain a string.
pub fn get_required_str(data: &Map<String, Value>, key: &str) -> Result<String, CallToolResult> {
    get_str(data, key).ok_or_else(|| {
        CallToolResult::error(vec![Content::text(format!(
            "Missing required field: {key}"
        ))])
    })
}

/// Extracts an i64 integer value from a JSON object by key.
///
/// Returns the integer value if the key exists and contains an i64, or `None` otherwise.
pub fn get_i64(data: &Map<String, Value>, key: &str) -> Option<i64> {
    data.get(key).and_then(|value| value.as_i64())
}

/// Extracts an i32 integer value from a JSON object by key.
///
/// Returns the integer value if the key exists and can be converted to i32, or `None` otherwise.
pub fn get_i32(data: &Map<String, Value>, key: &str) -> Option<i32> {
    data.get(key)
        .and_then(|value| value.as_i64())
        .and_then(|v| v.try_into().ok())
}

/// Extracts an f32 floating-point value from a JSON object by key.
///
/// Returns the floating-point value if the key exists and contains a number, or `None` otherwise.
pub fn get_f32(data: &Map<String, Value>, key: &str) -> Option<f32> {
    data.get(key)
        .and_then(|value| value.as_f64())
        .map(|v| v as f32)
}

/// Extracts a boolean value from a JSON object by key.
///
/// Returns the boolean value if the key exists and contains a boolean, or `None` otherwise.
pub fn get_bool(data: &Map<String, Value>, key: &str) -> Option<bool> {
    data.get(key).and_then(|value| value.as_bool())
}

/// Extracts a list of strings from a JSON object by key.
///
/// Returns a vector of strings extracted from a JSON array. Non-string items are filtered out.
/// Returns an empty vector if the key doesn't exist or is not an array.
pub fn get_string_list(data: &Map<String, Value>, key: &str) -> Vec<String> {
    data.get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}
