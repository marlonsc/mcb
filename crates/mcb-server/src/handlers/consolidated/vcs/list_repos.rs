use super::responses::ListRepositoriesResponse;
use crate::collection_mapping;
use crate::formatter::ResponseFormatter;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

pub async fn list_repositories() -> Result<CallToolResult, McpError> {
    let repositories = collection_mapping::list_collections()
        .map_err(|e| McpError::internal_error(format!("Failed to list collections: {e}"), None))?;
    let result = ListRepositoriesResponse {
        count: repositories.len(),
        repositories,
    };
    ResponseFormatter::json_success(&result)
}
