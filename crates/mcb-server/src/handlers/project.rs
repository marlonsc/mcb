//! Project handler for workflow operations.

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;

use crate::args::ProjectArgs;

/// handler for project workflow operations.
pub struct ProjectHandler;

impl Default for ProjectHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectHandler {
    /// Create a new ProjectHandler.
    pub fn new() -> Self {
        Self
    }

    /// Handle a project tool request.
    pub async fn handle(
        &self,
        _params: Parameters<ProjectArgs>,
    ) -> Result<CallToolResult, McpError> {
        // TODO: Implement actual project workflow logic
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            "Project tool called (implementation in progress)",
        )]))
    }
}
