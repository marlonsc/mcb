use super::responses::{IndexResult, repo_path};
use crate::args::VcsArgs;
use crate::formatter::ResponseFormatter;
use crate::vcs_repository_registry;
use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use std::path::Path;
use std::sync::Arc;

pub async fn index_repository(
    vcs_provider: &Arc<dyn VcsProvider>,
    args: &VcsArgs,
) -> Result<CallToolResult, McpError> {
    let path = match repo_path(args) {
        Ok(p) => p,
        Err(error_result) => return Ok(error_result),
    };
    let repo = match vcs_provider.open_repository(Path::new(&path)).await {
        Ok(repo) => repo,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to open repository: {e}"
            ))]));
        }
    };
    let branches = args
        .branches
        .clone()
        .unwrap_or_else(|| vec![repo.default_branch.clone()]);
    let mut total_files = 0;
    for branch in &branches {
        match vcs_provider.list_files(&repo, branch).await {
            Ok(files) => total_files += files.len(),
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to list files in branch {branch}: {e}"
                ))]));
            }
        }
    }
    let commits_indexed = if args.include_commits.unwrap_or(false) {
        let mut count = 0;
        for branch in &branches {
            if let Ok(commits) = vcs_provider.commit_history(&repo, branch, Some(1000)).await {
                count += commits.len();
            }
        }
        count
    } else {
        0
    };
    let result = IndexResult {
        repository_id: repo.id.to_string(),
        path: repo.path.to_string_lossy().to_string(),
        default_branch: repo.default_branch.clone(),
        branches_found: branches.clone(),
        total_files,
        commits_indexed,
    };
    let _ = vcs_repository_registry::record_repository(repo.id.as_str(), &repo.path);
    ResponseFormatter::json_success(&result)
}
