//! Tool Registry Module
//!
//! Manages tool definitions and schema generation for the MCP protocol.
//! This module centralizes all tool metadata to enable consistent tool listing.

use rmcp::ErrorData as McpError;
use rmcp::model::Tool;
use std::borrow::Cow;
use std::sync::Arc;

use crate::args::{
    AnalyzeComplexityArgs, ClearIndexArgs, CreateSessionSummaryArgs, GetIndexingStatusArgs,
    GetSessionSummaryArgs, GetValidationRulesArgs, IndexCodebaseArgs, ListValidatorsArgs,
    MemoryGetObservationsArgs, MemoryInjectContextArgs, MemoryTimelineArgs, SearchCodeArgs,
    SearchMemoriesArgs, StoreObservationArgs, ValidateArchitectureArgs, ValidateFileArgs,
};

/// Tool definitions for MCP protocol
pub struct ToolDefinitions;

impl ToolDefinitions {
    /// Get the index_codebase tool definition
    pub fn index_codebase() -> Result<Tool, McpError> {
        Self::create_tool(
            "index_codebase",
            "Index a codebase directory for semantic search using vector embeddings",
            schemars::schema_for!(IndexCodebaseArgs),
        )
    }

    /// Get the search_code tool definition
    pub fn search_code() -> Result<Tool, McpError> {
        Self::create_tool(
            "search_code",
            "Search for code using natural language queries",
            schemars::schema_for!(SearchCodeArgs),
        )
    }

    /// Get the get_indexing_status tool definition
    pub fn get_indexing_status() -> Result<Tool, McpError> {
        Self::create_tool(
            "get_indexing_status",
            "Get the current indexing status and statistics",
            schemars::schema_for!(GetIndexingStatusArgs),
        )
    }

    /// Get the clear_index tool definition
    pub fn clear_index() -> Result<Tool, McpError> {
        Self::create_tool(
            "clear_index",
            "Clear the search index for a collection",
            schemars::schema_for!(ClearIndexArgs),
        )
    }

    /// Get the validate_architecture tool definition
    pub fn validate_architecture() -> Result<Tool, McpError> {
        Self::create_tool(
            "validate_architecture",
            "Run architecture validation rules on a codebase to check for Clean Architecture, SOLID, and other code quality violations",
            schemars::schema_for!(ValidateArchitectureArgs),
        )
    }

    /// Get the validate_file tool definition
    pub fn validate_file() -> Result<Tool, McpError> {
        Self::create_tool(
            "validate_file",
            "Validate a single file against architecture rules",
            schemars::schema_for!(ValidateFileArgs),
        )
    }

    /// Get the list_validators tool definition
    pub fn list_validators() -> Result<Tool, McpError> {
        Self::create_tool(
            "list_validators",
            "List all available validators with their descriptions",
            schemars::schema_for!(ListValidatorsArgs),
        )
    }

    /// Get the get_validation_rules tool definition
    pub fn get_validation_rules() -> Result<Tool, McpError> {
        Self::create_tool(
            "get_validation_rules",
            "Get available validation rules, optionally filtered by category",
            schemars::schema_for!(GetValidationRulesArgs),
        )
    }

    /// Get the analyze_complexity tool definition
    pub fn analyze_complexity() -> Result<Tool, McpError> {
        Self::create_tool(
            "analyze_complexity",
            "Analyze code complexity metrics (cyclomatic, cognitive, maintainability) for a file",
            schemars::schema_for!(AnalyzeComplexityArgs),
        )
    }

    /// Get the store_observation tool definition
    pub fn store_observation() -> Result<Tool, McpError> {
        Self::create_tool(
            "store_observation",
            "Store an observation in the semantic memory",
            schemars::schema_for!(StoreObservationArgs),
        )
    }

    /// Get the search_memories tool definition
    pub fn search_memories() -> Result<Tool, McpError> {
        Self::create_tool(
            "search_memories",
            "Search observations in semantic memory using a natural language query",
            schemars::schema_for!(SearchMemoriesArgs),
        )
    }

    /// Get the get_session_summary tool definition
    pub fn get_session_summary() -> Result<Tool, McpError> {
        Self::create_tool(
            "get_session_summary",
            "Retrieve a summary for a specific session ID",
            schemars::schema_for!(GetSessionSummaryArgs),
        )
    }

    /// Get the create_session_summary tool definition
    pub fn create_session_summary() -> Result<Tool, McpError> {
        Self::create_tool(
            "create_session_summary",
            "Create or update a summary for a coding session",
            schemars::schema_for!(CreateSessionSummaryArgs),
        )
    }

    pub fn memory_timeline() -> Result<Tool, McpError> {
        Self::create_tool(
            "memory_timeline",
            "[EXPERIMENTAL] Step 2 of progressive disclosure: Get context around an anchor observation",
            schemars::schema_for!(MemoryTimelineArgs),
        )
    }

    pub fn memory_get_observations() -> Result<Tool, McpError> {
        Self::create_tool(
            "memory_get_observations",
            "[EXPERIMENTAL] Step 3 of progressive disclosure: Fetch full details for specific observation IDs",
            schemars::schema_for!(MemoryGetObservationsArgs),
        )
    }

    pub fn memory_inject_context() -> Result<Tool, McpError> {
        Self::create_tool(
            "memory_inject_context",
            "[EXPERIMENTAL] Generate context bundle for session start injection",
            schemars::schema_for!(MemoryInjectContextArgs),
        )
    }

    fn create_tool(
        name: &'static str,
        description: &'static str,
        schema: schemars::Schema,
    ) -> Result<Tool, McpError> {
        let schema_value = serde_json::to_value(schema)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let input_schema = schema_value
            .as_object()
            .ok_or_else(|| {
                McpError::internal_error(format!("Schema for {} is not an object", name), None)
            })?
            .clone();

        Ok(Tool {
            name: Cow::Borrowed(name),
            title: None,
            description: Some(Cow::Borrowed(description)),
            input_schema: Arc::new(input_schema),
            output_schema: None,
            annotations: None,
            icons: None,
            meta: Default::default(),
        })
    }
}

/// Create the complete list of available tools
///
/// Returns all tool definitions for the MCP list_tools response.
pub fn create_tool_list() -> Result<Vec<Tool>, McpError> {
    Ok(vec![
        ToolDefinitions::index_codebase()?,
        ToolDefinitions::search_code()?,
        ToolDefinitions::get_indexing_status()?,
        ToolDefinitions::clear_index()?,
        ToolDefinitions::validate_architecture()?,
        ToolDefinitions::validate_file()?,
        ToolDefinitions::list_validators()?,
        ToolDefinitions::get_validation_rules()?,
        ToolDefinitions::analyze_complexity()?,
        ToolDefinitions::store_observation()?,
        ToolDefinitions::search_memories()?,
        ToolDefinitions::get_session_summary()?,
        ToolDefinitions::create_session_summary()?,
        ToolDefinitions::memory_timeline()?,
        ToolDefinitions::memory_get_observations()?,
        ToolDefinitions::memory_inject_context()?,
    ])
}
