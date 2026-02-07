use mcb_domain::entities::memory::{ExecutionType, ObservationType, QualityGateStatus};
use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};

/// Helper utilities for memory handler operations.
///
/// Provides static methods for extracting, parsing, and validating data from JSON objects
/// used in memory-related MCP tool handlers.
pub struct MemoryHelpers;

impl MemoryHelpers {
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

    /// Extracts a required string value from a JSON object by key.
    ///
    /// Returns the string value if the key exists and contains a string.
    /// Returns an error result if the key is missing or does not contain a string.
    pub fn get_required_str(
        data: &Map<String, Value>,
        key: &str,
    ) -> Result<String, CallToolResult> {
        Self::get_str(data, key).ok_or_else(|| {
            CallToolResult::error(vec![Content::text(format!(
                "Missing required field: {key}"
            ))])
        })
    }

    /// Parses a string into an `ObservationType`.
    ///
    /// Accepts case-insensitive strings: "code", "decision", "context", "error", "summary", "execution", "quality_gate".
    /// Returns an error result if the string does not match any valid observation type.
    pub fn parse_observation_type(value: &str) -> Result<ObservationType, CallToolResult> {
        match value.to_lowercase().as_str() {
            "code" => Ok(ObservationType::Code),
            "decision" => Ok(ObservationType::Decision),
            "context" => Ok(ObservationType::Context),
            "error" => Ok(ObservationType::Error),
            "summary" => Ok(ObservationType::Summary),
            "execution" => Ok(ObservationType::Execution),
            "quality_gate" => Ok(ObservationType::QualityGate),
            _ => Err(CallToolResult::error(vec![Content::text(format!(
                "Unknown observation type: {value}"
            ))])),
        }
    }

    /// Parses a string into an `ExecutionType`.
    ///
    /// Returns an error result if the string does not match any valid execution type.
    pub fn parse_execution_type(value: &str) -> Result<ExecutionType, CallToolResult> {
        value.parse().map_err(|_| {
            CallToolResult::error(vec![Content::text(format!(
                "Unknown execution type: {value}"
            ))])
        })
    }

    /// Parses a string into a `QualityGateStatus`.
    ///
    /// Returns an error result if the string does not match any valid quality gate status.
    pub fn parse_quality_gate_status(value: &str) -> Result<QualityGateStatus, CallToolResult> {
        value.parse().map_err(|_| {
            CallToolResult::error(vec![Content::text(format!(
                "Unknown quality gate status: {value}"
            ))])
        })
    }
}
