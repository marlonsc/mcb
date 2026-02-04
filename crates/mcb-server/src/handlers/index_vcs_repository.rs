//! Handler for the `index_vcs_repository` MCP tool

use crate::args::IndexVcsRepositoryArgs;
use crate::formatter::ResponseFormatter;
use crate::vcs_repository_registry;
use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use validator::Validate;

/// Handler for the MCP `index_vcs_repository` tool that wires the repository indexing phase to Phase 6
/// by invoking the VCS provider, collecting branch/file/commit metrics, and returning a structured report.
pub struct IndexVcsRepositoryHandler {
    vcs_provider: Arc<dyn VcsProvider>,
}

#[derive(Serialize)]
struct IndexResult {
    repository_id: String,
    path: String,
    default_branch: String,
    branches_found: Vec<String>,
    total_files: usize,
    commits_indexed: usize,
}

impl IndexVcsRepositoryHandler {
    pub fn new(vcs_provider: Arc<dyn VcsProvider>) -> Self {
        Self { vcs_provider }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<IndexVcsRepositoryArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let path = Path::new(&args.path);

        let repo = match self.vcs_provider.open_repository(path).await {
            Ok(r) => r,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to open repository: {e}"
                ))]));
            }
        };

        let branches_to_index = if args.branches.is_empty() {
            vec![repo.default_branch.clone()]
        } else {
            args.branches.clone()
        };

        let mut total_files = 0;
        for branch in &branches_to_index {
            match self.vcs_provider.list_files(&repo, branch).await {
                Ok(files) => total_files += files.len(),
                Err(e) => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to list files in branch {branch}: {e}"
                    ))]));
                }
            }
        }

        let commits_indexed = if args.include_commits {
            let mut count = 0;
            for branch in &branches_to_index {
                if let Ok(commits) = self
                    .vcs_provider
                    .commit_history(&repo, branch, Some(1000))
                    .await
                {
                    count += commits.len();
                }
            }
            count
        } else {
            0
        };

        let result = IndexResult {
            repository_id: repo.id.as_str().to_string(),
            path: args.path,
            default_branch: repo.default_branch,
            branches_found: repo.branches,
            total_files,
            commits_indexed,
        };
        if let Err(e) = vcs_repository_registry::record_repository(repo.id.as_str(), &repo.path) {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to record repository mapping: {e}"
            ))]));
        }
        ResponseFormatter::json_success(&result)
    }
}
