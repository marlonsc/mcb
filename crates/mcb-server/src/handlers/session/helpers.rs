use mcb_domain::entities::agent::{AgentSessionStatus, AgentType};
use rmcp::ErrorData as McpError;

/// Helper utilities for session handler operations.
pub struct SessionHelpers;

impl SessionHelpers {
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
