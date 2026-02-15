use std::path::Path;
use std::sync::Arc;

use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::responses::{BranchSearchMatch, BranchSearchResponse, repo_path};
use crate::args::VcsArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handlers::helpers::tool_error;

/// Searches for a query string within a branch.
#[tracing::instrument(skip_all)]
pub async fn search_branch(
    vcs_provider: &Arc<dyn VcsProvider>,
    args: &VcsArgs,
) -> Result<CallToolResult, McpError> {
    let query = match args.query.as_ref() {
        Some(value) if !value.trim().is_empty() => value.trim(),
        _ => {
            return Ok(tool_error("Missing query for branch search"));
        }
    };
    let path = match repo_path(args) {
        Ok(p) => p,
        Err(error_result) => return Ok(error_result),
    };
    let repo = match vcs_provider.open_repository(Path::new(&path)).await {
        Ok(repo) => repo,
        Err(e) => {
            return Ok(to_contextual_tool_error(e));
        }
    };
    let branch = args
        .target_branch
        .clone()
        .unwrap_or_else(|| repo.default_branch().to_owned());
    let files = match vcs_provider.list_files(&repo, &branch).await {
        Ok(files) => files,
        Err(e) => {
            return Ok(to_contextual_tool_error(e));
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
                        path: file_path.to_str().unwrap_or_default().to_owned(),
                        line: index + 1,
                        snippet: line.trim().to_owned(),
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
        query: query.to_owned(),
        count: matches.len(),
        results: matches,
    };
    ResponseFormatter::json_success(&result)
}
