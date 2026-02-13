use mcb_domain::entities::memory::{ExecutionType, ObservationType, QualityGateStatus};
use rmcp::model::{CallToolResult, Content};

/// Helper utilities for memory handler operations.
///
/// Provides parsing and validation helpers for memory-specific enum values
/// used in memory-related MCP tool handlers.
pub struct MemoryHelpers;

impl MemoryHelpers {
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
