use std::path::PathBuf;
use std::sync::Arc;

use mcb_domain::ports::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::responses::ListRepositoriesResponse;
use crate::args::VcsArgs;
use crate::error_mapping::to_opaque_mcp_error;
use crate::formatter::ResponseFormatter;

/// Lists all available repositories discovered by the VCS provider.
#[tracing::instrument(skip_all)]
pub async fn list_repositories(
    vcs_provider: &Arc<dyn VcsProvider>,
    args: &VcsArgs,
) -> Result<CallToolResult, McpError> {
    let root = args
        .repo_path
        .as_ref()
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| {
            tracing::error!("no repo_path provided and current_dir() failed");
            McpError::invalid_params(
                "repo_path is required when working directory cannot be determined",
                None,
            )
        })?;

    let discovered_repos = vcs_provider
        .list_repositories(&root)
        .await
        .map_err(|e| to_opaque_mcp_error(&e))?;

    let repositories: Vec<String> = discovered_repos
        .iter()
        .map(|repo| repo.path().to_str().unwrap_or_default().to_owned())
        .collect();

    let result = ListRepositoriesResponse {
        count: repositories.len(),
        repositories,
    };
    ResponseFormatter::json_success(&result)
}
