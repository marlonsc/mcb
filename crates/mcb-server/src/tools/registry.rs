//!
//! Registry-backed tool definitions and dispatch for MCP protocol.

#![allow(unsafe_code)]

use std::borrow::Cow;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolRequestParams, CallToolResult, Tool};
use validator::Validate;

use crate::args::{
    AgentArgs, EntityArgs, IndexArgs, MemoryArgs, ProjectArgs, SearchArgs, SessionArgs,
    ValidateArgs, VcsArgs,
};
use crate::tools::router::ToolHandlers;

/// Async future returned by a descriptor-based tool call.
pub type ToolCallFuture<'a> =
    Pin<Box<dyn Future<Output = Result<CallToolResult, McpError>> + Send + 'a>>;
/// Function pointer signature used by registry-backed dispatch.
pub type ToolCallFn = for<'a> fn(&'a CallToolRequestParams, &'a ToolHandlers) -> ToolCallFuture<'a>;

/// Single source-of-truth descriptor for both tool listing and dispatch.
pub struct ToolDescriptor {
    /// MCP tool name.
    pub name: &'static str,
    /// Human-readable description surfaced in tool listing.
    pub description: &'static str,
    /// Schemars schema factory for this tool's input arguments.
    pub schema: fn() -> schemars::Schema,
    /// Async executor that preserves `handler.handle(Parameters(args)).await`.
    pub call: ToolCallFn,
}

#[linkme::distributed_slice]
/// Distributed slice containing all registered tool descriptors.
pub static TOOL_DESCRIPTORS: [ToolDescriptor];

fn schema_index() -> schemars::Schema {
    schemars::schema_for!(IndexArgs)
}

fn schema_search() -> schemars::Schema {
    schemars::schema_for!(SearchArgs)
}

fn schema_validate() -> schemars::Schema {
    schemars::schema_for!(ValidateArgs)
}

fn schema_memory() -> schemars::Schema {
    schemars::schema_for!(MemoryArgs)
}

fn schema_session() -> schemars::Schema {
    schemars::schema_for!(SessionArgs)
}

fn schema_agent() -> schemars::Schema {
    schemars::schema_for!(AgentArgs)
}

fn schema_project() -> schemars::Schema {
    schemars::schema_for!(ProjectArgs)
}

fn schema_vcs() -> schemars::Schema {
    schemars::schema_for!(VcsArgs)
}

fn schema_entity() -> schemars::Schema {
    schemars::schema_for!(EntityArgs)
}

fn parse_args<T>(request: &CallToolRequestParams) -> Result<T, McpError>
where
    T: serde::de::DeserializeOwned + Validate,
{
    let args_value = serde_json::Value::Object(request.arguments.clone().unwrap_or_default());
    let args: T = serde_json::from_value(args_value)
        .map_err(|e| McpError::invalid_params(format!("Failed to parse arguments: {e}"), None))?;

    args.validate()
        .map_err(|e| McpError::invalid_params(format!("Argument validation failed: {e}"), None))?;

    Ok(args)
}

fn call_index<'a>(
    request: &'a CallToolRequestParams,
    handlers: &'a ToolHandlers,
) -> ToolCallFuture<'a> {
    Box::pin(async move {
        let args = parse_args::<IndexArgs>(request)?;
        handlers.index.handle(Parameters(args)).await
    })
}

fn call_search<'a>(
    request: &'a CallToolRequestParams,
    handlers: &'a ToolHandlers,
) -> ToolCallFuture<'a> {
    Box::pin(async move {
        let args = parse_args::<SearchArgs>(request)?;
        handlers.search.handle(Parameters(args)).await
    })
}

fn call_validate<'a>(
    request: &'a CallToolRequestParams,
    handlers: &'a ToolHandlers,
) -> ToolCallFuture<'a> {
    Box::pin(async move {
        let args = parse_args::<ValidateArgs>(request)?;
        handlers.validate.handle(Parameters(args)).await
    })
}

fn call_memory<'a>(
    request: &'a CallToolRequestParams,
    handlers: &'a ToolHandlers,
) -> ToolCallFuture<'a> {
    Box::pin(async move {
        let args = parse_args::<MemoryArgs>(request)?;
        handlers.memory.handle(Parameters(args)).await
    })
}

fn call_session<'a>(
    request: &'a CallToolRequestParams,
    handlers: &'a ToolHandlers,
) -> ToolCallFuture<'a> {
    Box::pin(async move {
        let args = parse_args::<SessionArgs>(request)?;
        handlers.session.handle(Parameters(args)).await
    })
}

fn call_agent<'a>(
    request: &'a CallToolRequestParams,
    handlers: &'a ToolHandlers,
) -> ToolCallFuture<'a> {
    Box::pin(async move {
        let args = parse_args::<AgentArgs>(request)?;
        handlers.agent.handle(Parameters(args)).await
    })
}

fn call_project<'a>(
    request: &'a CallToolRequestParams,
    handlers: &'a ToolHandlers,
) -> ToolCallFuture<'a> {
    Box::pin(async move {
        let args = parse_args::<ProjectArgs>(request)?;
        handlers.project.handle(Parameters(args)).await
    })
}

fn call_vcs<'a>(
    request: &'a CallToolRequestParams,
    handlers: &'a ToolHandlers,
) -> ToolCallFuture<'a> {
    Box::pin(async move {
        let args = parse_args::<VcsArgs>(request)?;
        handlers.vcs.handle(Parameters(args)).await
    })
}

fn call_entity<'a>(
    request: &'a CallToolRequestParams,
    handlers: &'a ToolHandlers,
) -> ToolCallFuture<'a> {
    Box::pin(async move {
        let args = parse_args::<EntityArgs>(request)?;
        handlers.entity.handle(Parameters(args)).await
    })
}

#[linkme::distributed_slice(TOOL_DESCRIPTORS)]
static INDEX_DESCRIPTOR: ToolDescriptor = ToolDescriptor {
    name: "index",
    description: "Index operations (start, status, clear)",
    schema: schema_index,
    call: call_index,
};

#[linkme::distributed_slice(TOOL_DESCRIPTORS)]
static SEARCH_DESCRIPTOR: ToolDescriptor = ToolDescriptor {
    name: "search",
    description: "Search operations for code and memory",
    schema: schema_search,
    call: call_search,
};

#[linkme::distributed_slice(TOOL_DESCRIPTORS)]
static VALIDATE_DESCRIPTOR: ToolDescriptor = ToolDescriptor {
    name: "validate",
    description: "Validation and analysis operations",
    schema: schema_validate,
    call: call_validate,
};

#[linkme::distributed_slice(TOOL_DESCRIPTORS)]
static MEMORY_DESCRIPTOR: ToolDescriptor = ToolDescriptor {
    name: "memory",
    description: "Memory storage, retrieval, and timeline operations",
    schema: schema_memory,
    call: call_memory,
};

#[linkme::distributed_slice(TOOL_DESCRIPTORS)]
static SESSION_DESCRIPTOR: ToolDescriptor = ToolDescriptor {
    name: "session",
    description: "Session lifecycle operations",
    schema: schema_session,
    call: call_session,
};

#[linkme::distributed_slice(TOOL_DESCRIPTORS)]
static AGENT_DESCRIPTOR: ToolDescriptor = ToolDescriptor {
    name: "agent",
    description: "Agent activity logging operations",
    schema: schema_agent,
    call: call_agent,
};

#[linkme::distributed_slice(TOOL_DESCRIPTORS)]
static PROJECT_DESCRIPTOR: ToolDescriptor = ToolDescriptor {
    name: "project",
    description: "Project workflow management (phases, issues, dependencies, decisions)",
    schema: schema_project,
    call: call_project,
};

#[linkme::distributed_slice(TOOL_DESCRIPTORS)]
static VCS_DESCRIPTOR: ToolDescriptor = ToolDescriptor {
    name: "vcs",
    description: "Version control operations (list, index, compare, search, impact)",
    schema: schema_vcs,
    call: call_vcs,
};

#[linkme::distributed_slice(TOOL_DESCRIPTORS)]
static ENTITY_DESCRIPTOR: ToolDescriptor = ToolDescriptor {
    name: "entity",
    description: "Unified entity CRUD (vcs/plan/issue/org resources)",
    schema: schema_entity,
    call: call_entity,
};

fn create_tool_from_descriptor(descriptor: &ToolDescriptor) -> Result<Tool, McpError> {
    let schema_value = serde_json::to_value((descriptor.schema)())
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let input_schema = schema_value
        .as_object()
        .ok_or_else(|| {
            McpError::internal_error(
                format!("Schema for {} is not an object", descriptor.name),
                None,
            )
        })?
        .clone();

    Ok(Tool {
        name: Cow::Borrowed(descriptor.name),
        title: None,
        description: Some(Cow::Borrowed(descriptor.description)),
        input_schema: Arc::new(input_schema),
        output_schema: None,
        annotations: None,
        icons: None,
        execution: None,
        meta: Default::default(),
    })
}

fn validate_registry_unique_tool_names() -> Result<(), McpError> {
    let mut names = HashSet::new();
    for descriptor in TOOL_DESCRIPTORS {
        if !names.insert(descriptor.name) {
            return Err(McpError::internal_error(
                format!(
                    "Duplicate tool descriptor name detected: {}",
                    descriptor.name
                ),
                None,
            ));
        }
    }
    Ok(())
}

fn descriptor_by_name(name: &str) -> Option<&'static ToolDescriptor> {
    TOOL_DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.name == name)
}

/// Tool definitions for MCP protocol (compat wrapper over descriptor registry).
pub struct ToolDefinitions;

impl ToolDefinitions {
    /// Resolve a tool definition by name from the descriptor registry.
    pub fn by_name(name: &str) -> Result<Tool, McpError> {
        validate_registry_unique_tool_names()?;
        let descriptor = descriptor_by_name(name)
            .ok_or_else(|| McpError::invalid_params(format!("Unknown tool: {name}"), None))?;
        create_tool_from_descriptor(descriptor)
    }

    /// Define the `index` tool.
    pub fn index() -> Result<Tool, McpError> {
        Self::by_name("index")
    }

    /// Define the `search` tool.
    pub fn search() -> Result<Tool, McpError> {
        Self::by_name("search")
    }

    /// Define the `validate` tool.
    pub fn validate() -> Result<Tool, McpError> {
        Self::by_name("validate")
    }

    /// Define the `memory` tool.
    pub fn memory() -> Result<Tool, McpError> {
        Self::by_name("memory")
    }

    /// Define the `session` tool.
    pub fn session() -> Result<Tool, McpError> {
        Self::by_name("session")
    }

    /// Define the `agent` tool.
    pub fn agent() -> Result<Tool, McpError> {
        Self::by_name("agent")
    }

    /// Define the `project` tool.
    pub fn project() -> Result<Tool, McpError> {
        Self::by_name("project")
    }

    /// Define the `vcs` tool.
    pub fn vcs() -> Result<Tool, McpError> {
        Self::by_name("vcs")
    }

    /// Define the `entity` tool.
    pub fn entity() -> Result<Tool, McpError> {
        Self::by_name("entity")
    }
}

/// Create the complete list of available tools from the shared registry.
pub fn create_tool_list() -> Result<Vec<Tool>, McpError> {
    validate_registry_unique_tool_names()?;
    TOOL_DESCRIPTORS
        .iter()
        .map(create_tool_from_descriptor)
        .collect()
}

/// Dispatch to the tool call function from the shared descriptor registry.
pub async fn dispatch_tool_call(
    request: &CallToolRequestParams,
    handlers: &ToolHandlers,
) -> Result<CallToolResult, McpError> {
    validate_registry_unique_tool_names()?;
    let descriptor = descriptor_by_name(request.name.as_ref())
        .ok_or_else(|| McpError::invalid_params(format!("Unknown tool: {}", request.name), None))?;

    (descriptor.call)(request, handlers).await
}
