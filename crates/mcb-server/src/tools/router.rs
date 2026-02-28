//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//!
//! Routes incoming tool call requests to the appropriate handlers.
//! This module provides a centralized dispatch mechanism for MCP tool calls.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mcb_domain::ports::VcsProvider;
use mcb_domain::value_objects::ids::SessionId;
use mcb_domain::warn;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult, Meta};
use serde_json::Value;
use uuid::Uuid;

/// Canonical string field → alias mapping for context resolution.
///
/// Each entry maps a canonical key to all accepted aliases (`camelCase`, `x-header`, `snake_case`).
const STRING_FIELD_ALIASES: &[(&str, &[&str])] = &[
    (
        "session_id",
        &["session_id", "sessionId", "x-session-id", "x_session_id"],
    ),
    (
        "parent_session_id",
        &[
            "parent_session_id",
            "parentSessionId",
            "x-parent-session-id",
            "x_parent_session_id",
        ],
    ),
    (
        "project_id",
        &["project_id", "projectId", "x-project-id", "x_project_id"],
    ),
    (
        "worktree_id",
        &[
            "worktree_id",
            "worktreeId",
            "x-worktree-id",
            "x_worktree_id",
        ],
    ),
    ("repo_id", &["repo_id", "repoId", "x-repo-id", "x_repo_id"]),
    (
        "repo_path",
        &["repo_path", "repoPath", "x-repo-path", "x_repo_path"],
    ),
    (
        "workspace_root",
        &["workspace_root", "workspaceRoot", "x-workspace-root"],
    ),
    (
        "operator_id",
        &[
            "operator_id",
            "operatorId",
            "x-operator-id",
            "x_operator_id",
        ],
    ),
    (
        "machine_id",
        &["machine_id", "machineId", "x-machine-id", "x_machine_id"],
    ),
    (
        "agent_program",
        &[
            "agent_program",
            "agentProgram",
            "ide",
            "x-agent-program",
            "x_agent_program",
        ],
    ),
    (
        "model_id",
        &["model_id", "model", "modelId", "x-model-id", "x_model_id"],
    ),
    (
        "execution_flow",
        &[
            "execution_flow",
            "executionFlow",
            "x-execution-flow",
            "x_execution_flow",
        ],
    ),
];

/// Canonical boolean field → alias mapping for context resolution.
const BOOL_FIELD_ALIASES: &[(&str, &[&str])] = &[(
    "delegated",
    &["delegated", "is_delegated", "isDelegated", "x-delegated"],
)];

use crate::handlers::{
    AgentHandler, EntityHandler, IndexHandler, IssueEntityHandler, MemoryHandler, OrgEntityHandler,
    PlanEntityHandler, ProjectHandler, SearchHandler, SessionHandler, ValidateHandler,
    VcsEntityHandler, VcsHandler,
};
use crate::hooks::{HookProcessor, PostToolUseContext};
use crate::tools::dispatch_tool_call;

/// Handler references for tool routing
#[derive(Clone)]
pub struct ToolHandlers {
    /// Handler for coding implementation tasks.
    pub index: Arc<IndexHandler>,
    /// Handler for search operations.
    pub search: Arc<SearchHandler>,
    /// Handler for validation operations.
    pub validate: Arc<ValidateHandler>,
    /// Handler for memory operations.
    pub memory: Arc<MemoryHandler>,
    /// Handler for session management.
    pub session: Arc<SessionHandler>,
    /// Handler for agent operations.
    pub agent: Arc<AgentHandler>,
    /// Handler for project management.
    pub project: Arc<ProjectHandler>,
    /// Handler for VCS operations.
    pub vcs: Arc<VcsHandler>,
    /// Handler for VCS entity CRUD.
    pub vcs_entity: Arc<VcsEntityHandler>,
    /// Handler for plan entity CRUD.
    pub plan_entity: Arc<PlanEntityHandler>,
    /// Handler for issue entity CRUD.
    pub issue_entity: Arc<IssueEntityHandler>,
    /// Handler for org entity CRUD.
    pub org_entity: Arc<OrgEntityHandler>,
    /// Handler for unified entity CRUD.
    pub entity: Arc<EntityHandler>,
    /// Processor for tool execution hooks.
    pub hook_processor: Arc<HookProcessor>,
}

/// Valid execution flow modes for MCP tool dispatch.
///
/// Determines how the MCP server processes requests: direct stdio,
/// client-bridged to a running HTTP daemon, or server-managed hybrid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionFlow {
    /// Direct stdio transport with no HTTP server.
    StdioOnly,
    /// Client bridges stdio calls to a running HTTP server.
    ClientHybrid,
    /// Server manages both HTTP and background stdio transport.
    ServerHybrid,
}

impl ExecutionFlow {
    /// Wire-format string for this execution flow.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StdioOnly => "stdio-only",
            Self::ClientHybrid => "client-hybrid",
            Self::ServerHybrid => "server-hybrid",
        }
    }
}

impl std::fmt::Display for ExecutionFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ExecutionFlow {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "stdio-only" => Ok(Self::StdioOnly),
            "client-hybrid" => Ok(Self::ClientHybrid),
            "server-hybrid" => Ok(Self::ServerHybrid),
            other => Err(format!(
                "Invalid execution_flow '{other}'. Expected one of: {}, {}, {}",
                Self::StdioOnly.as_str(),
                Self::ClientHybrid.as_str(),
                Self::ServerHybrid.as_str(),
            )),
        }
    }
}

/// Boot-time execution provenance defaults used by context resolution.
#[derive(Debug, Clone)]
pub struct RuntimeDefaults {
    /// Workspace root discovered from the current working directory.
    pub workspace_root: Option<String>,
    /// Default repository path for tool execution.
    pub repo_path: Option<String>,
    /// Default repository identifier.
    pub repo_id: Option<String>,
    /// Default operator identifier.
    pub operator_id: Option<String>,
    /// Default machine identifier.
    pub machine_id: Option<String>,
    /// Default session identifier.
    pub session_id: Option<String>,
    /// Default agent program identifier.
    pub agent_program: Option<String>,
    /// Default model identifier.
    pub model_id: Option<String>,
    /// Default execution flow.
    pub execution_flow: Option<ExecutionFlow>,
}

impl RuntimeDefaults {
    /// Discover runtime defaults once at server boot.
    pub async fn discover(vcs: &dyn VcsProvider, execution_flow: Option<ExecutionFlow>) -> Self {
        let cwd = std::env::current_dir().ok();
        Self::discover_from_path(vcs, cwd.as_deref(), execution_flow).await
    }

    /// Discover runtime defaults from a given path.
    ///
    /// # Arguments
    /// * `vcs` - VCS provider for repository discovery
    /// * `cwd` - Current working directory (optional)
    /// * `execution_flow` - Execution flow mode (optional)
    ///
    /// # Returns
    /// `RuntimeDefaults` with discovered values
    pub async fn discover_from_path(
        vcs: &dyn VcsProvider,
        cwd: Option<&Path>,
        execution_flow: Option<ExecutionFlow>,
    ) -> Self {
        let workspace_root = match cwd {
            Some(path) => discover_workspace_root(vcs, path).await,
            None => None,
        };

        let repo_path = workspace_root.clone();
        let repo_id = if let Some(path) = workspace_root.as_deref() {
            vcs.open_repository(Path::new(path))
                .await
                .ok()
                .map(|repo| vcs.repository_id(&repo).into_string())
        } else {
            None
        };

        let machine_id = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .or_else(|| std::env::var("HOSTNAME").ok());

        Self {
            workspace_root,
            repo_path,
            repo_id,
            operator_id: std::env::var("USER").ok(),
            machine_id,
            session_id: Some(Uuid::new_v4().to_string()),
            agent_program: Some("mcb-stdio".to_owned()),
            model_id: Some("unknown".to_owned()),
            execution_flow,
        }
    }
}

async fn discover_workspace_root(vcs: &dyn VcsProvider, cwd: &Path) -> Option<String> {
    let mut discovered_root: Option<PathBuf> = None;

    for candidate in cwd.ancestors() {
        if vcs.open_repository(candidate).await.is_ok() {
            discovered_root = Some(candidate.to_path_buf());
            continue;
        }

        if discovered_root.is_some() {
            break;
        }
    }

    discovered_root.map(|path| path.to_string_lossy().into_owned())
}

#[derive(Debug, Clone, Default)]
/// Execution context extracted at transport boundary and propagated to hooks.
pub struct ToolExecutionContext {
    /// Canonical session identifier for the current tool call.
    pub session_id: Option<String>,
    /// Optional parent session identifier for delegated/subagent calls.
    pub parent_session_id: Option<String>,
    /// Optional project identifier associated with this execution.
    pub project_id: Option<String>,
    /// Optional worktree identifier associated with this execution.
    pub worktree_id: Option<String>,
    /// Optional repository identifier associated with this execution.
    pub repo_id: Option<String>,
    /// Optional repository/workspace path associated with this execution.
    pub repo_path: Option<String>,
    /// Optional operator/user identifier for this execution.
    pub operator_id: Option<String>,
    /// Optional machine/host fingerprint for this execution.
    pub machine_id: Option<String>,
    /// Optional agent program/IDE identifier for this execution.
    pub agent_program: Option<String>,
    /// Optional model identifier for this execution.
    pub model_id: Option<String>,
    /// Optional delegated flag for this execution.
    pub delegated: Option<bool>,
    /// Execution timestamp (Unix timestamp in seconds).
    pub timestamp: Option<i64>,
    /// Optional execution flow identifier for tracing.
    pub execution_flow: Option<String>,
}

impl ToolExecutionContext {
    /// Collect request/context metadata into canonical override keys.
    #[must_use]
    pub fn metadata_overrides(
        request_meta: Option<&Meta>,
        context_meta: &Meta,
    ) -> HashMap<String, String> {
        let mut overrides = HashMap::new();
        for &(canonical, aliases) in STRING_FIELD_ALIASES {
            insert_override(
                &mut overrides,
                canonical,
                resolve_context_value(request_meta, context_meta, aliases),
            );
        }
        for &(canonical, aliases) in BOOL_FIELD_ALIASES {
            if let Some(val) = resolve_context_bool(request_meta, context_meta, aliases) {
                overrides.insert(canonical.to_owned(), val.to_string());
            }
        }
        overrides
    }

    /// Resolve execution context from explicit overrides and runtime defaults.
    #[must_use]
    pub fn resolve(defaults: &RuntimeDefaults, overrides: &HashMap<String, String>) -> Self {
        let session_id = resolve_override_value(overrides, field_aliases("session_id"))
            .or_else(|| defaults.session_id.clone());
        let parent_session_id =
            resolve_override_value(overrides, field_aliases("parent_session_id"));
        let project_id = resolve_override_value(overrides, field_aliases("project_id"));
        let worktree_id = resolve_override_value(overrides, field_aliases("worktree_id"));
        let repo_id = resolve_override_value(overrides, field_aliases("repo_id"))
            .or_else(|| defaults.repo_id.clone());
        let repo_path = resolve_override_value(overrides, field_aliases("repo_path"))
            .or_else(|| resolve_override_value(overrides, field_aliases("workspace_root")))
            .or_else(|| defaults.repo_path.clone())
            .or_else(|| defaults.workspace_root.clone());
        let operator_id = resolve_override_value(overrides, field_aliases("operator_id"))
            .or_else(|| defaults.operator_id.clone());
        let machine_id = resolve_override_value(overrides, field_aliases("machine_id"))
            .or_else(|| defaults.machine_id.clone());
        let agent_program = resolve_override_value(overrides, field_aliases("agent_program"))
            .or_else(|| defaults.agent_program.clone());
        let model_id = resolve_override_value(overrides, field_aliases("model_id"))
            .or_else(|| defaults.model_id.clone());
        let delegated = resolve_override_bool(overrides, field_aliases("delegated"))
            .or(Some(parent_session_id.is_some()));
        let execution_flow = resolve_override_value(overrides, field_aliases("execution_flow"))
            .or_else(|| defaults.execution_flow.map(|f| f.to_string()));

        Self {
            session_id,
            parent_session_id,
            project_id,
            worktree_id,
            repo_id,
            repo_path,
            operator_id,
            machine_id,
            agent_program,
            model_id,
            delegated,
            timestamp: mcb_domain::utils::time::epoch_secs_i64().ok(),
            execution_flow,
        }
    }

    /// Inject execution context into tool arguments when those keys are missing.
    pub fn apply_to_request_if_missing(&self, request: &mut CallToolRequestParams) {
        let args = request.arguments.get_or_insert_with(Default::default);
        for (key, value) in [
            ("session_id", self.session_id.as_deref().map(str_value)),
            (
                "parent_session_id",
                self.parent_session_id.as_deref().map(str_value),
            ),
            ("project_id", self.project_id.as_deref().map(str_value)),
            ("worktree_id", self.worktree_id.as_deref().map(str_value)),
            ("repo_id", self.repo_id.as_deref().map(str_value)),
            ("repo_path", self.repo_path.as_deref().map(str_value)),
            ("operator_id", self.operator_id.as_deref().map(str_value)),
            ("machine_id", self.machine_id.as_deref().map(str_value)),
            (
                "agent_program",
                self.agent_program.as_deref().map(str_value),
            ),
            ("model_id", self.model_id.as_deref().map(str_value)),
            ("delegated", self.delegated.map(Value::Bool)),
            ("timestamp", self.timestamp.map(|v| Value::Number(v.into()))),
            (
                "execution_flow",
                self.execution_flow.as_deref().map(str_value),
            ),
        ] {
            if let Some(v) = value {
                args.entry(key.to_owned()).or_insert(v);
            }
        }
    }
}

fn str_value(s: &str) -> Value {
    Value::String(s.to_owned())
}

/// Look up aliases for a canonical field name from the alias tables.
fn field_aliases(canonical: &str) -> &'static [&'static str] {
    STRING_FIELD_ALIASES
        .iter()
        .chain(BOOL_FIELD_ALIASES.iter())
        .find(|&&(k, _)| k == canonical)
        .map_or(&[] as &[&str], |&(_, aliases)| aliases)
}

fn insert_override(overrides: &mut HashMap<String, String>, key: &str, value: Option<String>) {
    if let Some(value) = normalize_text(value) {
        overrides.insert(key.to_owned(), value);
    }
}

fn resolve_override_value(overrides: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = normalize_text(overrides.get(*key).cloned()) {
            return Some(value);
        }
    }
    None
}

fn resolve_override_bool(overrides: &HashMap<String, String>, keys: &[&str]) -> Option<bool> {
    for key in keys {
        let Some(raw) = overrides.get(*key) else {
            continue;
        };

        match raw.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" => return Some(true),
            "false" | "0" | "no" => return Some(false),
            _ => continue,
        }
    }

    None
}

fn normalize_text(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

fn meta_value_as_string(meta: &Meta, keys: &[&str]) -> Option<String> {
    for key in keys {
        let Some(value) = meta.get(*key) else {
            continue;
        };

        let extracted = match value {
            Value::String(v) => normalize_text(Some(v.clone())),
            Value::Number(v) => Some(v.to_string()),
            Value::Bool(v) => Some(v.to_string()),
            Value::Null | Value::Array(_) | Value::Object(_) => None,
        };

        if extracted.is_some() {
            return extracted;
        }
    }

    None
}

fn resolve_context_value(
    request_meta: Option<&Meta>,
    context_meta: &Meta,
    keys: &[&str],
) -> Option<String> {
    request_meta
        .and_then(|meta| meta_value_as_string(meta, keys))
        .or_else(|| meta_value_as_string(context_meta, keys))
}

fn meta_value_as_bool(meta: &Meta, keys: &[&str]) -> Option<bool> {
    for key in keys {
        let Some(value) = meta.get(*key) else {
            continue;
        };

        let extracted = match value {
            Value::Bool(v) => Some(*v),
            Value::String(v) => match v.trim().to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            },
            Value::Null | Value::Number(_) | Value::Array(_) | Value::Object(_) => None,
        };

        if extracted.is_some() {
            return extracted;
        }
    }

    None
}

fn resolve_context_bool(
    request_meta: Option<&Meta>,
    context_meta: &Meta,
    keys: &[&str],
) -> Option<bool> {
    request_meta
        .and_then(|meta| meta_value_as_bool(meta, keys))
        .or_else(|| meta_value_as_bool(context_meta, keys))
}

/// Route a tool call request to the appropriate handler
///
/// Parses the request arguments and delegates to the matching handler.
/// After tool execution, automatically triggers `PostToolUse` hook for memory operations.
///
/// # Errors
/// Returns an error when execution context validation or tool dispatch fails.
pub async fn route_tool_call(
    request: CallToolRequestParams,
    handlers: &ToolHandlers,
    execution_context: ToolExecutionContext,
) -> Result<CallToolResult, McpError> {
    validate_execution_context(request.name.as_ref(), &execution_context)?;

    let result = dispatch_tool_call(&request, handlers).await?;

    if let Err(e) = trigger_post_tool_use_hook(
        request.name.as_ref(),
        &result,
        &handlers.hook_processor,
        &execution_context,
    )
    .await
    {
        warn!("ToolRouter", "PostToolUse hook failed (non-fatal)", &e);
    }

    Ok(result)
}

/// Validate execution context for tool execution.
///
/// Checks operation mode matrix and provenance scope requirements.
///
/// # Arguments
/// * `tool_name` - Name of the tool being executed
/// * `execution_context` - Execution context to validate
///
/// # Errors
///
/// Returns `McpError` if the operation mode or provenance scope check fails.
pub fn validate_execution_context(
    tool_name: &str,
    execution_context: &ToolExecutionContext,
) -> Result<(), McpError> {
    validate_operation_mode_matrix(tool_name, execution_context)?;

    if !matches!(tool_name, "index" | "search" | "memory") {
        return Ok(());
    }

    let mut missing = Vec::new();
    for (key, value) in [
        ("session_id", &execution_context.session_id),
        ("repo_id", &execution_context.repo_id),
        ("repo_path", &execution_context.repo_path),
        ("operator_id", &execution_context.operator_id),
        ("machine_id", &execution_context.machine_id),
        ("agent_program", &execution_context.agent_program),
        ("model_id", &execution_context.model_id),
    ] {
        if is_missing_text(value) {
            missing.push(key);
        }
    }
    if execution_context.delegated.is_none() {
        missing.push("delegated");
    }
    if execution_context.timestamp.is_none() {
        missing.push("timestamp");
    }
    if execution_context.delegated == Some(true)
        && is_missing_text(&execution_context.parent_session_id)
    {
        missing.push("parent_session_id");
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(McpError::invalid_params(
            format!(
                "Missing execution provenance for '{tool_name}': {}",
                missing.join(", ")
            ),
            None,
        ))
    }
}

fn is_missing_text(value: &Option<String>) -> bool {
    value.as_deref().is_none_or(|s| s.trim().is_empty())
}

fn validate_operation_mode_matrix(
    tool_name: &str,
    execution_context: &ToolExecutionContext,
) -> Result<(), McpError> {
    let flow = normalize_execution_flow(execution_context.execution_flow.as_deref())?;

    let allowed: &[ExecutionFlow] = if matches!(tool_name, "validate") {
        &[ExecutionFlow::StdioOnly, ExecutionFlow::ClientHybrid]
    } else {
        &[
            ExecutionFlow::StdioOnly,
            ExecutionFlow::ClientHybrid,
            ExecutionFlow::ServerHybrid,
        ]
    };

    if allowed.contains(&flow) {
        Ok(())
    } else {
        Err(McpError::invalid_params(
            format!(
                "Operation mode matrix violation for '{tool_name}': flow '{}' is not allowed. Allowed flows: {}",
                flow,
                allowed
                    .iter()
                    .map(|f| f.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            None,
        ))
    }
}

fn normalize_execution_flow(flow: Option<&str>) -> Result<ExecutionFlow, McpError> {
    let raw = flow.unwrap_or(ExecutionFlow::StdioOnly.as_str());
    raw.parse::<ExecutionFlow>()
        .map_err(|e| McpError::invalid_params(e, None))
}

async fn trigger_post_tool_use_hook(
    tool_name: &str,
    result: &CallToolResult,
    hook_processor: &HookProcessor,
    execution_context: &ToolExecutionContext,
) -> Result<(), String> {
    let mut context =
        PostToolUseContext::new(tool_name.to_owned(), result.is_error.unwrap_or(false));

    if let Some(session_id) = &execution_context.session_id {
        context = context.with_session_id(SessionId::from_string(session_id));
    }
    for (key, value) in [
        (
            "parent_session_id",
            execution_context.parent_session_id.as_deref(),
        ),
        ("project_id", execution_context.project_id.as_deref()),
        ("worktree_id", execution_context.worktree_id.as_deref()),
        ("repo_id", execution_context.repo_id.as_deref()),
        ("repo_path", execution_context.repo_path.as_deref()),
        ("operator_id", execution_context.operator_id.as_deref()),
        ("machine_id", execution_context.machine_id.as_deref()),
        ("agent_program", execution_context.agent_program.as_deref()),
        ("model_id", execution_context.model_id.as_deref()),
    ] {
        if let Some(v) = value {
            context = context.with_metadata(key, v);
        }
    }
    if let Some(delegated) = execution_context.delegated {
        context = context.with_metadata("delegated", delegated.to_string());
    }
    if let Some(timestamp) = execution_context.timestamp {
        context = context.with_metadata("timestamp", timestamp.to_string());
    }

    hook_processor
        .process_post_tool_use(context)
        .await
        .map_err(|e| e.to_string())
}
