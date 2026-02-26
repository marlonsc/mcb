//! Registry-backed tool definitions and dispatch for MCP protocol.
// linkme distributed_slice uses #[link_section] internally
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
use crate::error_mapping::safe_internal_error;
use crate::tools::router::ToolHandlers;

/// Async future returned by a descriptor-based tool call.
pub type ToolCallFuture<'a> =
    Pin<Box<dyn Future<Output = Result<CallToolResult, McpError>> + Send + 'a>>;
/// Function pointer signature used by registry-backed dispatch.
pub type ToolCallFn = for<'a> fn(&'a CallToolRequestParams, &'a ToolHandlers) -> ToolCallFuture<'a>;

/// Single source-of-truth descriptor for both tool listing and dispatch.
pub struct ToolDescriptor {
    /// Unique tool name exposed via MCP.
    pub name: &'static str,
    /// Human-readable tool description.
    pub description: &'static str,
    /// Factory that produces the JSON schema for tool arguments.
    pub schema: fn() -> schemars::Schema,
    /// Dispatch function for tool invocation.
    pub call: ToolCallFn,
}

#[linkme::distributed_slice]
/// All registered tool descriptors.
pub static TOOL_DESCRIPTORS: [ToolDescriptor];

/// Register a tool: generates schema factory, dispatch function, and linkme descriptor.
macro_rules! register_tool {
    ($schema_fn:ident, $call_fn:ident, $descriptor:ident, $handler:ident, $args:ty, $name:literal, $desc:literal) => {
        fn $schema_fn() -> schemars::Schema {
            schemars::schema_for!($args)
        }
        fn $call_fn<'a>(
            request: &'a CallToolRequestParams,
            handlers: &'a ToolHandlers,
        ) -> ToolCallFuture<'a> {
            Box::pin(async move {
                let args = parse_args::<$args>(request)?;
                handlers.$handler.handle(Parameters(args)).await
            })
        }
        #[linkme::distributed_slice(TOOL_DESCRIPTORS)]
        static $descriptor: ToolDescriptor = ToolDescriptor {
            name: $name,
            description: $desc,
            schema: $schema_fn,
            call: $call_fn,
        };
    };
}

fn parse_args<T>(request: &CallToolRequestParams) -> Result<T, McpError>
where
    T: serde::de::DeserializeOwned + Validate,
{
    let args_value = match &request.arguments {
        Some(map) => serde_json::Value::Object(map.clone()),
        None => serde_json::Value::Object(Default::default()),
    };
    let args: T = serde_json::from_value(args_value)
        .map_err(|e| McpError::invalid_params(format!("Failed to parse arguments: {e}"), None))?;
    args.validate()
        .map_err(|e| McpError::invalid_params(format!("Argument validation failed: {e}"), None))?;
    Ok(args)
}

register_tool!(
    schema_index,
    call_index,
    INDEX_DESCRIPTOR,
    index,
    IndexArgs,
    "index",
    "Index operations (start, status, clear)"
);
register_tool!(
    schema_search,
    call_search,
    SEARCH_DESCRIPTOR,
    search,
    SearchArgs,
    "search",
    "Search operations for code and memory"
);
register_tool!(
    schema_validate,
    call_validate,
    VALIDATE_DESCRIPTOR,
    validate,
    ValidateArgs,
    "validate",
    "Validation and analysis operations"
);
register_tool!(
    schema_memory,
    call_memory,
    MEMORY_DESCRIPTOR,
    memory,
    MemoryArgs,
    "memory",
    "Memory storage, retrieval, and timeline operations"
);
register_tool!(
    schema_session,
    call_session,
    SESSION_DESCRIPTOR,
    session,
    SessionArgs,
    "session",
    "Session lifecycle operations"
);
register_tool!(
    schema_agent,
    call_agent,
    AGENT_DESCRIPTOR,
    agent,
    AgentArgs,
    "agent",
    "Agent activity logging operations"
);
register_tool!(
    schema_project,
    call_project,
    PROJECT_DESCRIPTOR,
    project,
    ProjectArgs,
    "project",
    "Project workflow management (phases, issues, dependencies, decisions)"
);
register_tool!(
    schema_vcs,
    call_vcs,
    VCS_DESCRIPTOR,
    vcs,
    VcsArgs,
    "vcs",
    "Version control operations (list, index, compare, search, impact)"
);
register_tool!(
    schema_entity,
    call_entity,
    ENTITY_DESCRIPTOR,
    entity,
    EntityArgs,
    "entity",
    "Unified entity CRUD (vcs/plan/issue/org resources)"
);

fn create_tool_from_descriptor(descriptor: &ToolDescriptor) -> Result<Tool, McpError> {
    let schema_value = serde_json::to_value((descriptor.schema)())
        .map_err(|e| safe_internal_error("serialize tool schema", &e))?;
    let input_schema = schema_value
        .as_object()
        .ok_or_else(|| {
            safe_internal_error(
                "validate tool schema shape",
                &format_args!("schema for '{}' is not an object", descriptor.name),
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
            return Err(safe_internal_error(
                "validate tool registry",
                &format_args!("duplicate tool descriptor: {}", descriptor.name),
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

/// Resolve a tool definition by name from the descriptor registry.
///
/// # Errors
///
/// Returns an error if the tool name is unknown or the registry contains duplicates.
pub fn tool_by_name(name: &str) -> Result<Tool, McpError> {
    validate_registry_unique_tool_names()?;
    let descriptor = descriptor_by_name(name)
        .ok_or_else(|| McpError::invalid_params(format!("Unknown tool: {name}"), None))?;
    create_tool_from_descriptor(descriptor)
}

/// Create the complete list of available tools from the shared registry.
///
/// # Errors
///
/// Returns an error if the registry contains duplicates or a schema fails to serialize.
pub fn create_tool_list() -> Result<Vec<Tool>, McpError> {
    validate_registry_unique_tool_names()?;
    TOOL_DESCRIPTORS
        .iter()
        .map(create_tool_from_descriptor)
        .collect()
}

/// Dispatch to the tool call function from the shared descriptor registry.
///
/// # Errors
///
/// Returns an error if the tool name is unknown or the handler fails.
pub async fn dispatch_tool_call(
    request: &CallToolRequestParams,
    handlers: &ToolHandlers,
) -> Result<CallToolResult, McpError> {
    validate_registry_unique_tool_names()?;
    let descriptor = descriptor_by_name(request.name.as_ref())
        .ok_or_else(|| McpError::invalid_params(format!("Unknown tool: {}", request.name), None))?;
    (descriptor.call)(request, handlers).await
}
