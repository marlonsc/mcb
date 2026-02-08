use std::path::Path;
use std::sync::Arc;

use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use super::responses::{BranchSearchMatch, BranchSearchResponse, repo_path};
use crate::args::VcsArgs;
use crate::formatter::ResponseFormatter;

/// Searches for a query string within a branch.
pub async fn search_branch(
    vcs_provider: &Arc<dyn VcsProvider>,
    args: &VcsArgs,
) -> Result<CallToolResult, McpError> {
    let query = match args.query.as_ref() {
        Some(value) if !value.trim().is_empty() => value.trim(),
        _ => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing query for branch search",
            )]));
        }
    };
    let branch = args
        .target_branch
        .clone()
        .unwrap_or_else(|| "main".to_string());
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
    let files = match vcs_provider.list_files(&repo, &branch).await {
        Ok(files) => files,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to list files: {e}"
            ))]));
        }
    };
    let limit = args.limit.unwrap_or(20) as usize;
    let mut matches = Vec::new();
    for file_path in files {
        if matches.len() >= limit {
            break;
        }
        if let Ok(content) = vcs_provider.read_file(&repo, &branch, &file_path).await {
            for (index, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&query.to_lowercase()) {
                    matches.push(BranchSearchMatch {
                        path: file_path.to_string_lossy().to_string(),
                        line: index + 1,
                        snippet: line.trim().to_string(),
                    });
                    if matches.len() >= limit {
                        break;
                    }
                }
            }
        }
    }
    let result = BranchSearchResponse {
        repository_id: repo.id().to_string(),
        branch,
        query: query.to_string(),
        count: matches.len(),
        results: matches,
    };
    ResponseFormatter::json_success(&result)
}
