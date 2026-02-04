//! Handler for the `list_repositories` MCP tool
//!
//! **Data flow**: Uses `collection_mapping::list_collections()` to return indexed
//! collection names (user-friendly names from the mapping). Collections represent
//! indexed codebases.

use crate::args::ListRepositoriesArgs;
use crate::collection_mapping;
use crate::formatter::ResponseFormatter;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use serde::Serialize;

/// Handler for `list_repositories` that returns indexed collection names.
pub struct ListRepositoriesHandler;

#[derive(Serialize)]
struct ListRepositoriesResponse {
    repositories: Vec<String>,
    count: usize,
}

impl ListRepositoriesHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        Parameters(_args): Parameters<ListRepositoriesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let repositories = collection_mapping::list_collections().map_err(|e| {
            McpError::internal_error(format!("Failed to list collections: {}", e), None)
        })?;
        let count = repositories.len();
        let result = ListRepositoriesResponse {
            repositories,
            count,
        };
        ResponseFormatter::json_success(&result)
    }
}

impl Default for ListRepositoriesHandler {
    fn default() -> Self {
        Self::new()
    }
}
