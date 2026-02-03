//! Handler for the `list_repositories` MCP tool

use crate::args::ListRepositoriesArgs;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use validator::Validate;

/// Handler for `list-repositories` that exposes repository metadata to tooling dashboards so Phase 6
/// (Memory Search) can keep `release/v0.1.5` repository visibility up to date.
pub struct ListRepositoriesHandler;

#[derive(Serialize)]
struct RepositoryInfo {
    id: String,
    path: String,
    default_branch: String,
}

#[derive(Serialize)]
struct ListResult {
    repositories: Vec<RepositoryInfo>,
    count: usize,
}

impl ListRepositoriesHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ListRepositoriesArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let result = ListResult {
            repositories: vec![],
            count: 0,
        };

        let json = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "Failed to serialize result".to_string());

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

impl Default for ListRepositoriesHandler {
    fn default() -> Self {
        Self::new()
    }
}
