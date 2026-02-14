use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::args::SessionArgs;
pub(super) use crate::handlers::shared::{
    opt_str, optional_data_map, require_data_map, require_str, str_vec,
};
use mcb_domain::value_objects::ids::SessionId;

pub(super) fn require_session_id(args: &SessionArgs) -> Result<&SessionId, CallToolResult> {
    args.session_id
        .as_ref()
        .ok_or_else(|| CallToolResult::error(vec![Content::text("Missing session_id")]))
}

pub(super) fn require_session_id_str(args: &SessionArgs) -> Result<&str, CallToolResult> {
    require_session_id(args).map(|id| id.as_str())
}

pub(super) fn parse_agent_type(
    value: &str,
) -> Result<mcb_domain::entities::agent::AgentType, McpError> {
    value
        .parse::<mcb_domain::entities::agent::AgentType>()
        .map_err(|e: String| McpError::invalid_params(e, None))
}
