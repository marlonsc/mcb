//! VCS handler implementation.

use std::sync::Arc;

use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use validator::Validate;

use super::{analyze_impact, compare_branches, index_repo, list_repos, search_branch};
use crate::args::{VcsAction, VcsArgs};

/// Handler for VCS-related MCP tool operations.
///
/// Supports listing, indexing, comparing, and searching repositories.
#[derive(Clone)]
pub struct VcsHandler {
    vcs_provider: Arc<dyn VcsProvider>,
}

impl VcsHandler {
    pub fn new(vcs_provider: Arc<dyn VcsProvider>) -> Self {
        Self { vcs_provider }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<VcsArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;

        match args.action {
            VcsAction::ListRepositories => list_repos::list_repositories().await,
            VcsAction::IndexRepository => {
                index_repo::index_repository(&self.vcs_provider, &args).await
            }
            VcsAction::CompareBranches => {
                compare_branches::compare_branches(&self.vcs_provider, &args).await
            }
            VcsAction::SearchBranch => {
                search_branch::search_branch(&self.vcs_provider, &args).await
            }
            VcsAction::AnalyzeImpact => {
                analyze_impact::analyze_impact(&self.vcs_provider, &args).await
            }
        }
    }
}
