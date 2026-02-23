//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use std::path::Path;
use std::sync::Arc;

use mcb_domain::ports::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::responses::{BranchSearchMatch, BranchSearchResponse, repo_path};
use crate::args::VcsArgs;
use crate::constants::limits::DEFAULT_VCS_SEARCH_LIMIT;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::utils::mcp::tool_error;

fn require_query(args: &VcsArgs) -> Result<&str, CallToolResult> {
    args.query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| tool_error("Missing query for branch search"))
}

fn append_file_matches(
    matches: &mut Vec<BranchSearchMatch>,
    file_path: &Path,
    content: &str,
    query_lower: &str,
    limit: usize,
) {
    for (index, line) in content.lines().enumerate() {
        if line.to_lowercase().contains(query_lower) {
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

/// Searches for a query string within a branch.
#[tracing::instrument(skip_all)]
pub async fn search_branch(
    vcs_provider: &Arc<dyn VcsProvider>,
    args: &VcsArgs,
) -> Result<CallToolResult, McpError> {
    let query = match require_query(args) {
        Ok(value) => value,
        Err(error_result) => return Ok(error_result),
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
    let limit = args.limit.unwrap_or(DEFAULT_VCS_SEARCH_LIMIT as u32) as usize;
    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();
    for file_path in files {
        if matches.len() >= limit {
            break;
        }
        if let Ok(content) = vcs_provider.read_file(&repo, &branch, &file_path).await {
            append_file_matches(&mut matches, &file_path, &content, &query_lower, limit);
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
