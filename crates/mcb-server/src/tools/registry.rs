//! Registry-backed tool definitions and dispatch for MCP protocol.
//!
//! # Safety
//!
//! [`linkme::distributed_slice`] uses `#[link_section]` internally which
//! requires `unsafe_code`. The macro generates only a static array of
//! [`ToolDescriptor`] references — no raw pointer dereference or mutable
//! aliasing occurs. Every entry is produced by [`register_tool!`] which emits
//! safe, read-only descriptors. This is the same pattern used in `mcb-domain`
//! registry macros. See: <https://docs.rs/linkme/latest/linkme/attr.distributed_slice.html>
//!
//! The `#![allow(unsafe_code)]` is scoped to this module file alone — it is
//! the single blessed location for tool registration in the server crate.

// Allow unsafe in this module only — linkme::distributed_slice requires it,
// and there is no safe alternative for compile-time registration.
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
    AgentArgs, AnalyzeCodeArgs, AnalyzeImpactArgs, ClearIndexArgs, CompareBranchesArgs, EntityArgs,
    GetMemoriesArgs, GetSessionArgs, IndexArgs, IndexRepoArgs, IndexStatusArgs, InjectContextArgs,
    ListMemoriesArgs, ListReposArgs, ListRulesArgs, ListSessionsArgs, LogDelegationArgs,
    LogToolCallArgs, MemoryArgs, MemoryTimelineArgs, ProjectArgs, SearchArgs, SearchCodeArgs,
    SearchMemoryArgs, SessionArgs, StartSessionArgs, StoreMemoryArgs, SummarizeSessionArgs,
    ValidateArgs, ValidateCodeArgs, VcsArgs,
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

// `register_tool!` macro is defined in `crate::macros::registry` and available via `#[macro_use]`.

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

// ---------------------------------------------------------------------------
// Search tools (mapped → SearchArgs)
// ---------------------------------------------------------------------------
register_tool!(
    schema_search_code, call_search_code, SEARCH_CODE_DESCRIPTOR,
    search, SearchCodeArgs => SearchArgs,
    "search_code",
    "Search for code in your project using natural language.\n\
     The repository is automatically detected and indexed.\n\
     If not yet indexed, indexing starts in the background.\n\n\
     Just describe what you're looking for:\n\
     - \"authentication middleware\"\n\
     - \"error handling in API routes\"\n\
     - \"database connection setup\"\n\n\
     Returns matching code snippets ranked by relevance,\n\
     with file path, line numbers, and programming language."
);
register_tool!(
    schema_search_memory, call_search_memory, SEARCH_MEMORY_DESCRIPTOR,
    search, SearchMemoryArgs => SearchArgs,
    "search_memory",
    "Search through stored memories and observations.\n\
     Finds previously stored knowledge, decisions, patterns,\n\
     and context using semantic similarity.\n\n\
     Use natural language to describe what you remember:\n\
     - \"database migration decision\"\n\
     - \"error pattern for timeouts\"\n\n\
     Filter by tags for precision. Returns ranked matches\n\
     with content, timestamps, and metadata."
);

// ---------------------------------------------------------------------------
// Index tools (mapped → IndexArgs)
// ---------------------------------------------------------------------------
register_tool!(
    schema_index_repo, call_index_repo, INDEX_REPO_DESCRIPTOR,
    index, IndexRepoArgs => IndexArgs,
    "index_repo",
    "Index (or re-index) the current repository for code search.\n\
     The repository path is detected automatically.\n\n\
     Scans source files, generates embeddings, and stores them\n\
     for fast semantic search. Supports filtering by extension,\n\
     excluding directories, and setting file size limits.\n\n\
     Run once per repo; re-run after major changes."
);
register_tool!(
    schema_index_status, call_index_status, INDEX_STATUS_DESCRIPTOR,
    index, IndexStatusArgs => IndexArgs,
    "index_status",
    "Check the current indexing status.\n\
     Returns whether indexing is in progress, complete, or idle,\n\
     along with file counts and timestamps."
);
register_tool!(
    schema_clear_index, call_clear_index, CLEAR_INDEX_DESCRIPTOR,
    index, ClearIndexArgs => IndexArgs,
    "clear_index",
    "Clear the search index for the current repository.\n\
     Removes all indexed embeddings. You will need to re-index\n\
     before code search works again."
);

// ---------------------------------------------------------------------------
// Memory tools (mapped → MemoryArgs)
// ---------------------------------------------------------------------------
register_tool!(
    schema_store_memory, call_store_memory, STORE_MEMORY_DESCRIPTOR,
    memory, StoreMemoryArgs => MemoryArgs,
    "store_memory",
    "Store a new observation or piece of knowledge.\n\
     Persists information across sessions so it can be\n\
     retrieved later via search or timeline.\n\n\
     Provide content as plain text or structured JSON:\n\
     {content, type?, tags?, metadata?}\n\n\
     Use tags to categorize for easier retrieval."
);
register_tool!(
    schema_get_memories, call_get_memories, GET_MEMORIES_DESCRIPTOR,
    memory, GetMemoriesArgs => MemoryArgs,
    "get_memories",
    "Retrieve specific memory items by their IDs.\n\
     Returns full content and metadata for each requested item."
);
register_tool!(
    schema_list_memories, call_list_memories, LIST_MEMORIES_DESCRIPTOR,
    memory, ListMemoriesArgs => MemoryArgs,
    "list_memories",
    "List and filter stored memories.\n\
     Supports filtering by tags, text query, and time window.\n\
     Returns a paginated list of matching observations."
);
register_tool!(
    schema_memory_timeline, call_memory_timeline, MEMORY_TIMELINE_DESCRIPTOR,
    memory, MemoryTimelineArgs => MemoryArgs,
    "memory_timeline",
    "View a chronological timeline of memories around an anchor point.\n\
     Centers on a specific observation and shows items before/after\n\
     to provide temporal context for decisions and events."
);
register_tool!(
    schema_inject_context, call_inject_context, INJECT_CONTEXT_DESCRIPTOR,
    memory, InjectContextArgs => MemoryArgs,
    "inject_context",
    "Inject relevant memories into the current context.\n\
     Automatically selects the most relevant observations\n\
     within a token budget for context enrichment.\n\
     Useful at session start to prime the agent with\n\
     prior knowledge."
);

// ---------------------------------------------------------------------------
// Session tools (mapped → SessionArgs)
// ---------------------------------------------------------------------------
register_tool!(
    schema_start_session, call_start_session, START_SESSION_DESCRIPTOR,
    session, StartSessionArgs => SessionArgs,
    "start_session",
    "Start a new agent session.\n\
     Creates a session record to track tool usage, decisions,\n\
     and delegations. Returns a session ID for subsequent calls.\n\n\
     Provide the AI model name and optional agent type."
);
register_tool!(
    schema_get_session, call_get_session, GET_SESSION_DESCRIPTOR,
    session, GetSessionArgs => SessionArgs,
    "get_session",
    "Retrieve details of an existing session by ID.\n\
     Returns session metadata, status, and associated data."
);
register_tool!(
    schema_list_sessions, call_list_sessions, LIST_SESSIONS_DESCRIPTOR,
    session, ListSessionsArgs => SessionArgs,
    "list_sessions",
    "List available sessions with optional filters.\n\
     Filter by status, agent type, or limit the result count.\n\
     Returns session summaries sorted by recency."
);
register_tool!(
    schema_summarize_session, call_summarize_session, SUMMARIZE_SESSION_DESCRIPTOR,
    session, SummarizeSessionArgs => SessionArgs,
    "summarize_session",
    "Generate a summary of a session's activity.\n\
     Produces a structured overview of tool calls, decisions,\n\
     delegations, and outcomes for the given session."
);

// ---------------------------------------------------------------------------
// Agent tools (mapped → AgentArgs)
// ---------------------------------------------------------------------------
register_tool!(
    schema_log_tool_call, call_log_tool_call, LOG_TOOL_CALL_DESCRIPTOR,
    agent, LogToolCallArgs => AgentArgs,
    "log_tool_call",
    "Log a tool execution event for the current session.\n\
     Records tool name, parameters summary, success/failure,\n\
     error message, and duration for observability and replay."
);
register_tool!(
    schema_log_delegation, call_log_delegation, LOG_DELEGATION_DESCRIPTOR,
    agent, LogDelegationArgs => AgentArgs,
    "log_delegation",
    "Log a delegation event (spawning a child agent).\n\
     Records the child session ID, prompt, result, success,\n\
     and duration for tracing multi-agent workflows."
);

// ---------------------------------------------------------------------------
// Validate tools (mapped → ValidateArgs)
// ---------------------------------------------------------------------------
register_tool!(
    schema_validate_code, call_validate_code, VALIDATE_CODE_DESCRIPTOR,
    validate, ValidateCodeArgs => ValidateArgs,
    "validate_code",
    "Run architectural validation rules against the codebase.\n\
     Checks layer violations, circular dependencies, naming\n\
     conventions, and other configurable rules.\n\n\
     Optionally filter by scope (file/project), specific rules,\n\
     or rule category."
);
register_tool!(
    schema_analyze_code, call_analyze_code, ANALYZE_CODE_DESCRIPTOR,
    validate, AnalyzeCodeArgs => ValidateArgs,
    "analyze_code",
    "Analyze code complexity metrics.\n\
     Computes cyclomatic complexity, cognitive complexity,\n\
     and other metrics for the specified path.\n\
     Useful for identifying hotspots and refactoring targets."
);
register_tool!(
    schema_list_rules, call_list_rules, LIST_RULES_DESCRIPTOR,
    validate, ListRulesArgs => ValidateArgs,
    "list_rules",
    "List available validation rules.\n\
     Shows all configured architecture and code quality rules\n\
     with descriptions. Optionally filter by category."
);

// ---------------------------------------------------------------------------
// VCS tools (mapped → VcsArgs)
// ---------------------------------------------------------------------------
register_tool!(
    schema_list_repos, call_list_repos, LIST_REPOS_DESCRIPTOR,
    vcs, ListReposArgs => VcsArgs,
    "list_repos",
    "List all repositories tracked by the project.\n\
     Returns repository metadata including ID, path, and\n\
     default branch information."
);
register_tool!(
    schema_compare_branches, call_compare_branches, COMPARE_BRANCHES_DESCRIPTOR,
    vcs, CompareBranchesArgs => VcsArgs,
    "compare_branches",
    "Compare two branches and show their differences.\n\
     Returns a diff summary between the base and target branches,\n\
     optionally including commit history and depth control."
);
register_tool!(
    schema_analyze_impact, call_analyze_impact, ANALYZE_IMPACT_DESCRIPTOR,
    vcs, AnalyzeImpactArgs => VcsArgs,
    "analyze_impact",
    "Analyze the impact of changes across branches.\n\
     Examines which files and modules are affected by changes,\n\
     helping assess risk and scope of modifications."
);

// ---------------------------------------------------------------------------
// Compound tools (direct dispatch, kept as-is)
// ---------------------------------------------------------------------------
register_tool!(
    schema_project,
    call_project,
    PROJECT_DESCRIPTOR,
    project,
    ProjectArgs,
    "project",
    "Project workflow management.\n\
     CRUD operations for project resources: phases, issues,\n\
     dependencies, and decisions.\n\n\
     Specify action (create/get/update/list/delete) and\n\
     resource type, plus a data payload for mutations."
);
register_tool!(
    schema_entity,
    call_entity,
    ENTITY_DESCRIPTOR,
    entity,
    EntityArgs,
    "entity",
    "Unified entity CRUD for all resource types.\n\
     Manages VCS (repos, branches, worktrees, assignments),\n\
     plans (plans, versions, reviews), issues (issues, comments,\n\
     labels), and org (orgs, users, teams, API keys).\n\n\
     Specify action + resource, with optional data payload."
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
    // rmcp 1.x marks Tool #[non_exhaustive]; build via its constructor.
    Ok(Tool::new(
        Cow::Borrowed(descriptor.name),
        Cow::Borrowed(descriptor.description),
        Arc::new(input_schema),
    ))
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
