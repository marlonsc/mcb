use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents a single tool execution within an agent session.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolCall {
    /// Unique identifier for the tool call.
    pub id: String,
    /// Session identifier this call belongs to.
    pub session_id: String,
    /// Name of the tool invoked.
    pub tool_name: String,
    /// Summary of parameters passed to the tool.
    pub params_summary: Option<String>,
    /// Whether the tool execution was successful.
    pub success: bool,
    /// Error message if the tool failed.
    pub error_message: Option<String>,
    /// Time taken to execute the tool in milliseconds.
    pub duration_ms: Option<i64>,
    /// Unix timestamp when the tool call was recorded.
    pub created_at: i64,
}
