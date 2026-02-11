use std::path::Path;
use std::sync::Arc;

use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::responses::{BranchComparison, BranchDiffFile, repo_path};
use crate::args::VcsArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;

/// Compares two branches and returns the diff.
#[tracing::instrument(skip_all)]
pub async fn compare_branches(
    vcs_provider: &Arc<dyn VcsProvider>,
    args: &VcsArgs,
) -> Result<CallToolResult, McpError> {
    let path = match repo_path(args) {
        Ok(p) => p,
        Err(error_result) => return Ok(error_result),
    };
    let base = args
        .base_branch
        .clone()
        .unwrap_or_else(|| "main".to_string());
    let head = args
        .target_branch
        .clone()
        .unwrap_or_else(|| "HEAD".to_string());
    let repo = match vcs_provider.open_repository(Path::new(&path)).await {
        Ok(repo) => repo,
        Err(e) => {
            return Ok(to_contextual_tool_error(e));
        }
    };
    let diff = match vcs_provider.diff_refs(&repo, &base, &head).await {
        Ok(diff) => diff,
        Err(e) => {
            return Ok(to_contextual_tool_error(e));
        }
    };
    let files = diff
        .files
        .iter()
        .map(|file| BranchDiffFile {
            path: file.path.to_string_lossy().to_string(),
            status: file.status.to_string(),
        })
        .collect();
    let result = BranchComparison {
        base_branch: base,
        head_branch: head,
        files_changed: diff.files.len(),
        additions: diff.total_additions,
        deletions: diff.total_deletions,
        files,
    };
    ResponseFormatter::json_success(&result)
}
