use std::path::PathBuf;
use std::sync::Arc;

use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::responses::ListRepositoriesResponse;
use crate::args::VcsArgs;
use crate::formatter::ResponseFormatter;

/// Lists all available repositories discovered by the VCS provider.
pub async fn list_repositories(
    vcs_provider: &Arc<dyn VcsProvider>,
    args: &VcsArgs,
) -> Result<CallToolResult, McpError> {
    let root = args
        .repo_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let discovered_repos = vcs_provider
        .list_repositories(&root)
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to list repositories: {e}"), None))?;

    let repositories: Vec<String> = discovered_repos
        .iter()
        .map(|repo| repo.path().to_string_lossy().to_string())
        .collect();

    let result = ListRepositoriesResponse {
        count: repositories.len(),
        repositories,
    };
    ResponseFormatter::json_success(&result)
}
