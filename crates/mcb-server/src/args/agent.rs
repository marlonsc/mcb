use mcb_domain::value_objects::ids::SessionId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::macros::{tool_enum, tool_schema};

tool_enum! {
pub enum AgentAction {
    LogTool,
    LogDelegation,
}
}

tool_schema! {
pub struct AgentArgs {
    #[schemars(description = "Action: log_tool, log_delegation")]
    pub action: AgentAction,

    #[schemars(description = "Organization ID (uses default if omitted)")]
    pub org_id: Option<String>,

    #[schemars(description = "Session ID for the agent")]
    pub session_id: SessionId,

    #[schemars(
        description = "Activity data payload. log_tool: {tool_name, params_summary?, success, error_message?, duration_ms?}; log_delegation: {child_session_id, prompt, prompt_embedding_id?, result?, success, duration_ms?}"
    )]
    pub data: serde_json::Value,
}
}
