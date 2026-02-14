use mcb_domain::value_objects::ids::SessionId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::macros::{tool_enum, tool_schema};

tool_enum! {
/// Actions available for agent activity logging
pub enum AgentAction {
    /// Log a tool execution.
    LogTool,
    /// Log a delegation event.
    LogDelegation,
}
}

tool_schema! {
/// Arguments for agent activity logging operations
pub struct AgentArgs {
    /// Action: log_tool, log_delegation
    #[schemars(description = "Action: log_tool, log_delegation")]
    pub action: AgentAction,

    /// Organization ID (uses default if omitted)
    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    /// Session ID for the agent
    #[schemars(description = "Session ID for the agent")]
    pub session_id: SessionId,

    /// Activity data payload. log_tool: {tool_name, params_summary?, success, error_message?, duration_ms?}; log_delegation: {child_session_id, prompt, prompt_embedding_id?, result?, success, duration_ms?}
    #[schemars(
        description = "Activity data payload. log_tool: {tool_name, params_summary?, success, error_message?, duration_ms?}; log_delegation: {child_session_id, prompt, prompt_embedding_id?, result?, success, duration_ms?}"
    )]
    pub data: serde_json::Value,
}
}
