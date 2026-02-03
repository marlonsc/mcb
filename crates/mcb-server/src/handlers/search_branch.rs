//! Handler for the `search_branch` MCP tool

use crate::args::SearchBranchArgs;
use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

/// Handler for the MCP `search_branch` tool (VCS branch-scoped code search).
pub struct SearchBranchHandler {
    // TODO: Use `vcs_provider` to perform actual branch search against the VCS backend
    /// VCS provider; reserved for future branch-scoped search (handler currently returns stub).
    #[allow(dead_code)] // Reserved for future branch-scoped search
    vcs_provider: Arc<dyn VcsProvider>,
}

/// Response shape for the search_branch tool (avoids duplicate name with domain SearchResult).
#[derive(Serialize)]
struct SearchBranchResponse {
    repository_id: String,
    branch: String,
    query: String,
    files_searched: usize,
    message: String,
}

impl SearchBranchHandler {
    pub fn new(vcs_provider: Arc<dyn VcsProvider>) -> Self {
        Self { vcs_provider }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<SearchBranchArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let result = SearchBranchResponse {
            repository_id: args.repository_id.clone(),
            branch: args.branch.clone(),
            query: args.query.clone(),
            files_searched: 0,
            message: format!(
                "Search in branch '{}' is ready. Repository ID: {}. Query: '{}'. Limit: {}.",
                args.branch, args.repository_id, args.query, args.limit
            ),
        };

        let json = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| String::from("Failed to serialize result"));

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}
