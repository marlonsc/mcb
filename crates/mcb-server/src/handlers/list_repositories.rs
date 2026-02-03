//! Handler for the `list_repositories` MCP tool
//!
//! Listing indexed repositories is not yet implemented; this handler returns an honest
//! "not implemented" response so clients do not treat an empty list as real data.

use crate::args::ListRepositoriesArgs;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;

/// Handler for `list_repositories` that will expose repository metadata to tooling.
///
/// Until a backend is wired, the handler returns a structured "not implemented" response.
pub struct ListRepositoriesHandler;

#[derive(Serialize)]
struct ListRepositoriesNotImplementedResponse {
    implemented: bool,
    message: String,
}

impl ListRepositoriesHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        Parameters(_args): Parameters<ListRepositoriesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let result = ListRepositoriesNotImplementedResponse {
            implemented: false,
            message: "List repositories is not implemented yet. No repository registry is wired."
                .to_string(),
        };

        let json = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| String::from("Failed to serialize result"));

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

impl Default for ListRepositoriesHandler {
    fn default() -> Self {
        Self::new()
    }
}
