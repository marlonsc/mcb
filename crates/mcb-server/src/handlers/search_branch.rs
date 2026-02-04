//! Handler for the `search_branch` MCP tool
//!
//! **Data flow**: The handler receives `VcsProvider` (port) from the composition root.
//! It resolves the repository path from the VCS registry, lists files in the branch,
//! reads content, and performs a simple substring search.

use crate::args::SearchBranchArgs;
use crate::formatter::ResponseFormatter;
use crate::vcs_repository_registry;
use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

/// Handler for the MCP `search_branch` tool (VCS branch-scoped code search).
///
/// The VCS provider is used to list and read files for a basic branch-scoped search.
pub struct SearchBranchHandler {
    vcs_provider: Arc<dyn VcsProvider>,
}

/// Single search hit within a branch.
#[derive(Serialize)]
struct BranchSearchMatch {
    path: String,
    line: usize,
    snippet: String,
}

/// Response shape for the search_branch tool.
#[derive(Serialize)]
struct SearchBranchResponse {
    repository_id: String,
    branch: String,
    query: String,
    count: usize,
    results: Vec<BranchSearchMatch>,
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

        let query = args.query.trim();
        if query.is_empty() {
            return Err(McpError::invalid_params("query cannot be empty", None));
        }

        let repo_path = match vcs_repository_registry::lookup_repository_path(&args.repository_id) {
            Ok(path) => path,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Repository not found: {e}. Run index_vcs_repository first."
                ))]));
            }
        };

        let repo = match self.vcs_provider.open_repository(&repo_path).await {
            Ok(r) => r,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to open repository: {e}"
                ))]));
            }
        };

        let files = match self.vcs_provider.list_files(&repo, &args.branch).await {
            Ok(list) => list,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to list files in branch {}: {e}",
                    args.branch
                ))]));
            }
        };

        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        'files: for path in files {
            if results.len() >= args.limit {
                break;
            }
            let content = match self
                .vcs_provider
                .read_file(&repo, &args.branch, &path)
                .await
            {
                Ok(text) => text,
                Err(_) => continue,
            };

            for (idx, line) in content.lines().enumerate() {
                if results.len() >= args.limit {
                    break 'files;
                }
                if line.to_lowercase().contains(&query_lower) {
                    results.push(BranchSearchMatch {
                        path: path.display().to_string(),
                        line: idx + 1,
                        snippet: line.trim_end().to_string(),
                    });
                }
            }
        }

        let result = SearchBranchResponse {
            repository_id: args.repository_id,
            branch: args.branch,
            query: query.to_string(),
            count: results.len(),
            results,
        };

        ResponseFormatter::json_success(&result)
    }
}
