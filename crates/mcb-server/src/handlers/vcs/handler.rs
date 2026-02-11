//! VCS handler implementation.

use std::sync::Arc;

use mcb_domain::ports::providers::VcsProvider;
use mcb_domain::value_objects::OrgContext;
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
    /// Create a new VcsHandler.
    pub fn new(vcs_provider: Arc<dyn VcsProvider>) -> Self {
        Self { vcs_provider }
    }

    /// Handle a VCS tool request.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<VcsArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("invalid arguments", None))?;

        let org_ctx = OrgContext::default();
        let _org_id = args.org_id.as_deref().unwrap_or(org_ctx.org_id.as_str());

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
}
