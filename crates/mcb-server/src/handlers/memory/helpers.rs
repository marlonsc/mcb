use mcb_domain::entities::memory::{ExecutionType, ObservationType, QualityGateStatus};
use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};

use crate::utils::json;

/// Helper utilities for memory handler operations.
///
/// Provides static methods for extracting, parsing, and validating data from JSON objects
/// used in memory-related MCP tool handlers.
pub struct MemoryHelpers;

impl MemoryHelpers {
    /// Extract a JSON map from an optional value.
    pub fn json_map(data: &Option<Value>) -> Option<&Map<String, Value>> {
        json::json_map(data)
    }

    /// Extract a string value from a map.
    pub fn get_str(data: &Map<String, Value>, key: &str) -> Option<String> {
        json::get_str(data, key)
    }

    /// Extract an i64 value from a map.
    pub fn get_i64(data: &Map<String, Value>, key: &str) -> Option<i64> {
        json::get_i64(data, key)
    }

    /// Extract an i32 value from a map.
    pub fn get_i32(data: &Map<String, Value>, key: &str) -> Option<i32> {
        json::get_i32(data, key)
    }

    /// Extract an f32 value from a map.
    pub fn get_f32(data: &Map<String, Value>, key: &str) -> Option<f32> {
        json::get_f32(data, key)
    }

    /// Extract a boolean value from a map.
    pub fn get_bool(data: &Map<String, Value>, key: &str) -> Option<bool> {
        json::get_bool(data, key)
    }

    /// Extract a list of strings from a map.
    pub fn get_string_list(data: &Map<String, Value>, key: &str) -> Vec<String> {
        json::get_string_list(data, key)
    }

    /// Extract a required string value from a map.
    pub fn get_required_str(
        data: &Map<String, Value>,
        key: &str,
    ) -> Result<String, CallToolResult> {
        json::get_required_str(data, key)
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
                "Unknown observation type: {value}. Valid types: code, decision, context, error, summary, execution, quality_gate"
            ))])),
        }
    }

    /// Parses a string into an `ExecutionType`.
    ///
    /// Returns an error result if the string does not match any valid execution type.
    pub fn parse_execution_type(value: &str) -> Result<ExecutionType, CallToolResult> {
        value
            .parse()
            .map_err(|e: String| CallToolResult::error(vec![Content::text(e)]))
    }

    /// Parse a quality gate status string, returning a contextual error on invalid input.
    pub fn parse_quality_gate_status(value: &str) -> Result<QualityGateStatus, CallToolResult> {
        value
            .parse()
            .map_err(|e: String| CallToolResult::error(vec![Content::text(e)]))
    }
}
