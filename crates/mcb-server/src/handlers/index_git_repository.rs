//! Handler for the `index_git_repository` MCP tool

use crate::args::IndexGitRepositoryArgs;
use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use validator::Validate;

pub struct IndexGitRepositoryHandler {
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

impl IndexGitRepositoryHandler {
    pub fn new(vcs_provider: Arc<dyn VcsProvider>) -> Self {
        Self { vcs_provider }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<IndexGitRepositoryArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

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

        let json = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "Failed to serialize result".to_string());

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}
