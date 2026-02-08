//!
//! Manages tool definitions and schema generation for the MCP protocol.
//! This module centralizes all tool metadata to enable consistent tool listing.

use std::borrow::Cow;
use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::model::Tool;

use crate::args::{
    AgentArgs, IndexArgs, MemoryArgs, ProjectArgs, SearchArgs, SessionArgs, ValidateArgs, VcsArgs,
};

/// Tool definitions for MCP protocol
pub struct ToolDefinitions;

impl ToolDefinitions {
    /// Define the `index` tool.
    pub fn index() -> Result<Tool, McpError> {
        Self::create_tool(
            "index",
            "Index operations (start, status, clear)",
            schemars::schema_for!(IndexArgs),
        )
    }

    /// Define the `search` tool.
    pub fn search() -> Result<Tool, McpError> {
        Self::create_tool(
            "search",
            "Search operations for code and memory",
            schemars::schema_for!(SearchArgs),
        )
    }

    /// Define the `validate` tool.
    pub fn validate() -> Result<Tool, McpError> {
        Self::create_tool(
            "validate",
            "Validation and analysis operations",
            schemars::schema_for!(ValidateArgs),
        )
    }

    /// Define the `memory` tool.
    pub fn memory() -> Result<Tool, McpError> {
        Self::create_tool(
            "memory",
            "Memory storage, retrieval, and timeline operations",
            schemars::schema_for!(MemoryArgs),
        )
    }

    /// Define the `session` tool.
    pub fn session() -> Result<Tool, McpError> {
        Self::create_tool(
            "session",
            "Session lifecycle operations",
            schemars::schema_for!(SessionArgs),
        )
    }

    /// Define the `agent` tool.
    pub fn agent() -> Result<Tool, McpError> {
        Self::create_tool(
            "agent",
            "Agent activity logging operations",
            schemars::schema_for!(AgentArgs),
        )
    }

    /// Define the `project` tool.
    pub fn project() -> Result<Tool, McpError> {
        Self::create_tool(
            "project",
            "Project workflow management (phases, issues, dependencies, decisions)",
            schemars::schema_for!(ProjectArgs),
        )
    }

    /// Define the `vcs` tool.
    pub fn vcs() -> Result<Tool, McpError> {
        Self::create_tool(
            "vcs",
            "Version control operations (list, index, compare, search, impact)",
            schemars::schema_for!(VcsArgs),
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
        ToolDefinitions::index()?,
        ToolDefinitions::search()?,
        ToolDefinitions::validate()?,
        ToolDefinitions::memory()?,
        ToolDefinitions::session()?,
        ToolDefinitions::agent()?,
        ToolDefinitions::project()?,
        ToolDefinitions::vcs()?,
    ])
}
