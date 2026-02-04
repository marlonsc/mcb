//! Tool Registry Module
//!
//! Manages tool definitions and schema generation for the MCP protocol.
//! This module centralizes all tool metadata to enable consistent tool listing.

use rmcp::ErrorData as McpError;
use rmcp::model::Tool;
use std::borrow::Cow;
use std::sync::Arc;

use crate::args::{
    AnalyzeComplexityArgs, ClearIndexArgs, CreateAgentSessionArgs, CreateSessionSummaryArgs,
    GetAgentSessionArgs, GetIndexingStatusArgs, GetSessionSummaryArgs, GetValidationRulesArgs,
    IndexCodebaseArgs, ListAgentSessionsArgs, ListValidatorsArgs, MemoryGetExecutionsArgs,
    MemoryGetObservationsArgs, MemoryGetQualityGatesArgs, MemoryInjectContextArgs,
    MemorySearchArgs, MemoryStoreExecutionArgs, MemoryStoreQualityGateArgs, MemoryTimelineArgs,
    SearchCodeArgs, SearchMemoriesArgs, StoreDelegationArgs, StoreObservationArgs,
    StoreToolCallArgs, UpdateAgentSessionArgs, ValidateArchitectureArgs, ValidateFileArgs,
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

    pub fn memory_search() -> Result<Tool, McpError> {
        Self::create_tool(
            "memory_search",
            "[EXPERIMENTAL] Step 1 of progressive disclosure: Token-efficient memory search (index only). Use with memory_get_observations for full details.",
            schemars::schema_for!(MemorySearchArgs),
        )
    }

    pub fn memory_store_execution() -> Result<Tool, McpError> {
        Self::create_tool(
            "memory_store_execution",
            "Store execution results in semantic memory",
            schemars::schema_for!(MemoryStoreExecutionArgs),
        )
    }

    pub fn memory_get_executions() -> Result<Tool, McpError> {
        Self::create_tool(
            "memory_get_executions",
            "Retrieve execution history with optional filters",
            schemars::schema_for!(MemoryGetExecutionsArgs),
        )
    }

    pub fn memory_store_quality_gate() -> Result<Tool, McpError> {
        Self::create_tool(
            "memory_store_quality_gate",
            "Store quality gate results in semantic memory",
            schemars::schema_for!(MemoryStoreQualityGateArgs),
        )
    }

    pub fn memory_get_quality_gates() -> Result<Tool, McpError> {
        Self::create_tool(
            "memory_get_quality_gates",
            "Retrieve quality gate results with optional filters",
            schemars::schema_for!(MemoryGetQualityGatesArgs),
        )
    }

    pub fn create_agent_session() -> Result<Tool, McpError> {
        Self::create_tool(
            "create_agent_session",
            "Create a new agent session record for tracking agent workflows",
            schemars::schema_for!(CreateAgentSessionArgs),
        )
    }

    pub fn get_agent_session() -> Result<Tool, McpError> {
        Self::create_tool(
            "get_agent_session",
            "Get details of an agent session by ID",
            schemars::schema_for!(GetAgentSessionArgs),
        )
    }

    pub fn update_agent_session() -> Result<Tool, McpError> {
        Self::create_tool(
            "update_agent_session",
            "Update an agent session (status, results, metrics)",
            schemars::schema_for!(UpdateAgentSessionArgs),
        )
    }

    pub fn list_agent_sessions() -> Result<Tool, McpError> {
        Self::create_tool(
            "list_agent_sessions",
            "List agent sessions with optional filtering",
            schemars::schema_for!(ListAgentSessionsArgs),
        )
    }

    pub fn store_tool_call() -> Result<Tool, McpError> {
        Self::create_tool(
            "store_tool_call",
            "Store a tool call record for an agent session",
            schemars::schema_for!(StoreToolCallArgs),
        )
    }

    pub fn store_delegation() -> Result<Tool, McpError> {
        Self::create_tool(
            "store_delegation",
            "Store a delegation record between agent sessions",
            schemars::schema_for!(StoreDelegationArgs),
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
        ToolDefinitions::memory_search()?,
        ToolDefinitions::memory_store_execution()?,
        ToolDefinitions::memory_get_executions()?,
        ToolDefinitions::memory_store_quality_gate()?,
        ToolDefinitions::memory_get_quality_gates()?,
        ToolDefinitions::create_agent_session()?,
        ToolDefinitions::get_agent_session()?,
        ToolDefinitions::update_agent_session()?,
        ToolDefinitions::list_agent_sessions()?,
        ToolDefinitions::store_tool_call()?,
        ToolDefinitions::store_delegation()?,
    ])
}
