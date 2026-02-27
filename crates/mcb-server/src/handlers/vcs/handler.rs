//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! VCS handler implementation.

use std::sync::Arc;

use mcb_domain::ports::VcsProvider;
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

handler_new!(VcsHandler {
    vcs_provider: Arc<dyn VcsProvider>,
});

impl VcsHandler {
    /// Handle a VCS tool request.
    ///
    /// # Errors
    /// Returns an error when argument validation fails.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<VcsArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("invalid vcs arguments: {e}"), None))?;

        Self::validate_action_params(&args)?;

        match args.action {
            VcsAction::ListRepositories => {
                list_repos::list_repositories(&self.vcs_provider, &args).await
            }
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

    /// Validates that required parameters are present for the requested action.
    fn validate_action_params(args: &VcsArgs) -> Result<(), McpError> {
        let has_repo_path = args
            .repo_path
            .as_ref()
            .is_some_and(|p| !p.trim().is_empty());

        match args.action {
            VcsAction::IndexRepository | VcsAction::CompareBranches | VcsAction::AnalyzeImpact => {
                if !has_repo_path {
                    return Err(McpError::invalid_params(
                        format!("repo_path is required for {:?}", args.action),
                        None,
                    ));
                }
            }
            VcsAction::SearchBranch => {
                if !has_repo_path {
                    return Err(McpError::invalid_params(
                        "repo_path is required for search_branch",
                        None,
                    ));
                }
                if args.query.as_ref().is_none_or(|q| q.trim().is_empty()) {
                    return Err(McpError::invalid_params(
                        "query is required for search_branch",
                        None,
                    ));
                }
            }
            VcsAction::ListRepositories => {}
        }
        Ok(())
    }
}
