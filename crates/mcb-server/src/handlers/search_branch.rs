//! Handler for the `search_branch` MCP tool
//!
//! Branch-scoped search is not yet implemented; this handler returns an honest
//! "not implemented" response so clients do not treat it as a real search result.

use crate::args::SearchBranchArgs;
use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

/// Handler for the MCP `search_branch` tool (VCS branch-scoped code search).
///
/// The VCS provider is retained for a future implementation. Until then,
/// the handler returns a structured "not implemented" response.
pub struct SearchBranchHandler {
    vcs_provider: Arc<dyn VcsProvider>,
}

/// Response shape for the search_branch tool when the feature is not implemented.
#[derive(Serialize)]
struct SearchBranchNotImplementedResponse {
    implemented: bool,
    repository_id: String,
    branch: String,
    query: String,
    message: String,
}

impl SearchBranchHandler {
    pub fn new(vcs_provider: Arc<dyn VcsProvider>) -> Self {
        Self { vcs_provider }
    }

    /// VCS provider for future branch-scoped search implementation
    pub fn vcs_provider(&self) -> Arc<dyn VcsProvider> {
        self.vcs_provider.clone()
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<SearchBranchArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let result = SearchBranchNotImplementedResponse {
            implemented: false,
            repository_id: args.repository_id.clone(),
            branch: args.branch.clone(),
            query: args.query.clone(),
            message:
                "Branch-scoped search is not implemented yet. Use search_code for workspace search."
                    .to_string(),
        };

        let json = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| String::from("Failed to serialize result"));

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}
