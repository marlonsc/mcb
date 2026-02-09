use std::sync::Arc;

use mcb_domain::ports::providers::VcsProvider;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::responses::ListRepositoriesResponse;
use crate::formatter::ResponseFormatter;
use crate::vcs_repository_registry;

/// Lists all available repositories.
pub async fn list_repositories(
    _vcs_provider: &Arc<dyn VcsProvider>,
) -> Result<CallToolResult, McpError> {
    let repos = vcs_repository_registry::list_repositories()
        .map_err(|e| McpError::internal_error(format!("Failed to list repositories: {e}"), None))?;

    let repositories: Vec<String> = repos
        .into_iter()
        .map(|(id, path)| format!("{} ({})", id.as_str(), path.display()))
        .collect();

    let result = ListRepositoriesResponse {
        count: repositories.len(),
        repositories,
    };
    ResponseFormatter::json_success(&result)
}
