//! Tool Router Module
//!
//! Routes incoming tool call requests to the appropriate handlers.
//! This module provides a centralized dispatch mechanism for MCP tool calls.

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use std::sync::Arc;

use crate::args::{
    AnalyzeComplexityArgs, ClearIndexArgs, GetIndexingStatusArgs, GetValidationRulesArgs,
    IndexCodebaseArgs, IndexGitRepositoryArgs, ListRepositoriesArgs, ListValidatorsArgs,
    SearchBranchArgs, SearchCodeArgs, ValidateArchitectureArgs, ValidateFileArgs,
};
use crate::handlers::{
    AnalyzeComplexityHandler, ClearIndexHandler, GetIndexingStatusHandler,
    GetValidationRulesHandler, IndexCodebaseHandler, IndexGitRepositoryHandler,
    ListRepositoriesHandler, ListValidatorsHandler, SearchBranchHandler, SearchCodeHandler,
    ValidateArchitectureHandler, ValidateFileHandler,
};

/// Handler references for tool routing
#[derive(Clone)]
pub struct ToolHandlers {
    /// Handler for codebase indexing operations
    pub index_codebase: Arc<IndexCodebaseHandler>,
    /// Handler for code search operations
    pub search_code: Arc<SearchCodeHandler>,
    /// Handler for indexing status operations
    pub get_indexing_status: Arc<GetIndexingStatusHandler>,
    /// Handler for index clearing operations
    pub clear_index: Arc<ClearIndexHandler>,
    /// Handler for architecture validation operations
    pub validate_architecture: Arc<ValidateArchitectureHandler>,
    /// Handler for single file validation
    pub validate_file: Arc<ValidateFileHandler>,
    /// Handler for listing validators
    pub list_validators: Arc<ListValidatorsHandler>,
    /// Handler for getting validation rules
    pub get_validation_rules: Arc<GetValidationRulesHandler>,
    /// Handler for complexity analysis
    pub analyze_complexity: Arc<AnalyzeComplexityHandler>,
    /// Handler for git repository indexing
    pub index_git_repository: Arc<IndexGitRepositoryHandler>,
    /// Handler for branch-specific search
    pub search_branch: Arc<SearchBranchHandler>,
    /// Handler for listing indexed repositories
    pub list_repositories: Arc<ListRepositoriesHandler>,
}

/// Route a tool call request to the appropriate handler
///
/// Parses the request arguments and delegates to the matching handler.
pub async fn route_tool_call(
    request: CallToolRequestParams,
    handlers: &ToolHandlers,
) -> Result<CallToolResult, McpError> {
    match request.name.as_ref() {
        "index_codebase" => {
            let args = parse_args::<IndexCodebaseArgs>(&request)?;
            handlers.index_codebase.handle(Parameters(args)).await
        }
        "search_code" => {
            let args = parse_args::<SearchCodeArgs>(&request)?;
            handlers.search_code.handle(Parameters(args)).await
        }
        "get_indexing_status" => {
            let args = parse_args::<GetIndexingStatusArgs>(&request)?;
            handlers.get_indexing_status.handle(Parameters(args)).await
        }
        "clear_index" => {
            let args = parse_args::<ClearIndexArgs>(&request)?;
            handlers.clear_index.handle(Parameters(args)).await
        }
        "validate_architecture" => {
            let args = parse_args::<ValidateArchitectureArgs>(&request)?;
            handlers
                .validate_architecture
                .handle(Parameters(args))
                .await
        }
        "validate_file" => {
            let args = parse_args::<ValidateFileArgs>(&request)?;
            handlers.validate_file.handle(Parameters(args)).await
        }
        "list_validators" => {
            let args = parse_args::<ListValidatorsArgs>(&request)?;
            handlers.list_validators.handle(Parameters(args)).await
        }
        "get_validation_rules" => {
            let args = parse_args::<GetValidationRulesArgs>(&request)?;
            handlers.get_validation_rules.handle(Parameters(args)).await
        }
        "analyze_complexity" => {
            let args = parse_args::<AnalyzeComplexityArgs>(&request)?;
            handlers.analyze_complexity.handle(Parameters(args)).await
        }
        "index_git_repository" => {
            let args = parse_args::<IndexGitRepositoryArgs>(&request)?;
            handlers.index_git_repository.handle(Parameters(args)).await
        }
        "search_branch" => {
            let args = parse_args::<SearchBranchArgs>(&request)?;
            handlers.search_branch.handle(Parameters(args)).await
        }
        "list_repositories" => {
            let args = parse_args::<ListRepositoriesArgs>(&request)?;
            handlers.list_repositories.handle(Parameters(args)).await
        }
        _ => Err(McpError::invalid_params(
            format!("Unknown tool: {}", request.name),
            None,
        )),
    }
}

/// Parse request arguments into the expected type
fn parse_args<T: serde::de::DeserializeOwned>(
    request: &CallToolRequestParams,
) -> Result<T, McpError> {
    let args_value = serde_json::Value::Object(request.arguments.clone().unwrap_or_default());
    serde_json::from_value(args_value)
        .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {}", e), None))
}
