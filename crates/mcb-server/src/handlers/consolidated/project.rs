//! Project handler for project workflow operations.

use crate::args::ProjectArgs;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use validator::Validate;

/// Handler for project workflow MCP tool operations.
#[derive(Default)]
pub struct ProjectHandler;

impl ProjectHandler {
    pub async fn handle(
        &self,
        Parameters(args): Parameters<ProjectArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;
        Ok(CallToolResult::error(vec![Content::text(
            "Project workflow is not implemented yet",
        )]))
    }
}
