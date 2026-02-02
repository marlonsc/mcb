//! Handler for the `compare_branches` MCP tool

use crate::args::CompareBranchesArgs;
use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use validator::Validate;

pub struct CompareBranchesHandler {
    vcs_provider: Arc<dyn VcsProvider>,
}

#[derive(Serialize)]
struct BranchComparison {
    base_branch: String,
    head_branch: String,
    files_changed: usize,
    additions: usize,
    deletions: usize,
    files: Vec<FileChange>,
}

#[derive(Serialize)]
struct FileChange {
    path: String,
    status: String,
}

impl CompareBranchesHandler {
    pub fn new(vcs_provider: Arc<dyn VcsProvider>) -> Self {
        Self { vcs_provider }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<CompareBranchesArgs>,
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

        let diff = match self
            .vcs_provider
            .diff_refs(&repo, &args.base_branch, &args.head_branch)
            .await
        {
            Ok(d) => d,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to compare branches: {e}"
                ))]));
            }
        };

        let files: Vec<FileChange> = diff
            .files
            .iter()
            .map(|f| FileChange {
                path: f.path.to_string_lossy().to_string(),
                status: format!("{:?}", f.status),
            })
            .collect();

        let result = BranchComparison {
            base_branch: args.base_branch,
            head_branch: args.head_branch,
            files_changed: diff.files.len(),
            additions: diff.total_additions,
            deletions: diff.total_deletions,
            files,
        };

        let json = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "Failed to serialize result".to_string());

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}
