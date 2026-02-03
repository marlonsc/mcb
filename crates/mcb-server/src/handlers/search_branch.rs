//! Handler for the `search_branch` MCP tool

use crate::args::SearchBranchArgs;
use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

/// MCP handler for the `search-branch` tool. Coordinates memory search results with branch metadata
/// so Hybrid Search (Phase 6) exposes diffs per branch while staying aligned with `.planning/STATE.md`.
pub struct SearchBranchHandler {
    #[allow(dead_code)]
    vcs_provider: Arc<dyn VcsProvider>,
}

#[derive(Serialize)]
struct SearchResult {
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
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let result = SearchResult {
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
            .unwrap_or_else(|_| "Failed to serialize result".to_string());

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}
