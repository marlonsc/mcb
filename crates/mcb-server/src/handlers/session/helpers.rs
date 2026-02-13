use mcb_domain::entities::agent::{AgentSessionStatus, AgentType};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::{Map, Value};

/// Helper utilities for session handler operations.
pub struct SessionHelpers;

impl SessionHelpers {
    /// Extract a JSON map from an optional value.
    pub fn json_map(data: &Option<Value>) -> Option<&Map<String, Value>> {
        crate::utils::json::json_map(data)
    }

    /// Extract a string value from a map.
    pub fn get_str(data: &Map<String, Value>, key: &str) -> Option<String> {
        crate::utils::json::get_str(data, key)
    }

    /// Extract an i64 value from a map.
    pub fn get_i64(data: &Map<String, Value>, key: &str) -> Option<i64> {
        crate::utils::json::get_i64(data, key)
    }

    /// Extract a boolean value from a map.
    pub fn get_bool(data: &Map<String, Value>, key: &str) -> Option<bool> {
        crate::utils::json::get_bool(data, key)
    }

    /// Extract a list of strings from a map.
    pub fn get_string_list(data: &Map<String, Value>, key: &str) -> Vec<String> {
        crate::utils::json::get_string_list(data, key)
    }

    /// Extract a required string value from a map.
    pub fn get_required_str(
        data: &Map<String, Value>,
        key: &str,
    ) -> Result<String, CallToolResult> {
        crate::utils::json::get_required_str(data, key)
    }

    /// Parse an agent type string, returning an MCP error listing valid types on failure.
    pub fn parse_agent_type(value: &str) -> Result<AgentType, McpError> {
        value
            .parse()
            .map_err(|e: String| McpError::invalid_params(e, None))
    }

    /// Parse an agent session status string, returning an MCP error on invalid input.
    pub fn parse_status(value: &str) -> Result<AgentSessionStatus, McpError> {
        value
            .parse()
            .map_err(|e: String| McpError::invalid_params(e, None))
    }
}
