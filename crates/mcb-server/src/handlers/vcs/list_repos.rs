//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
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
            mcb_domain::error!(
                "list_repositories",
                "no repo_path provided and current_dir() failed"
            );
            McpError::invalid_params(
                "repo_path is required when working directory cannot be determined",
                None,
            )
        })?;

    let discovered_repos = vcs_provider
        .list_repositories(&root)
        .await
        .map_err(|e| to_opaque_mcp_error(&e))?;

    let mut repositories: Vec<String> = discovered_repos
        .iter()
        // INTENTIONAL: Path to_str conversion; non-UTF8 paths yield empty string
        .map(|repo| repo.path().to_str().unwrap_or_default().to_owned())
        .collect();

    // Sort alphabetically for deterministic output across environments.
    repositories.sort();

    // Apply limit if specified.
    if let Some(limit) = args.limit {
        repositories.truncate(limit as usize);
    }

    let result = ListRepositoriesResponse {
        count: repositories.len(),
        repositories,
    };
    ResponseFormatter::json_success(&result)
}
