//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//!
//! Routes incoming tool call requests to the appropriate handlers.
//! This module provides a centralized dispatch mechanism for MCP tool calls.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mcb_domain::constants::keys as schema;
use mcb_domain::ports::VcsProvider;
use mcb_domain::value_objects::ids::SessionId;
use mcb_domain::warn;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult, Meta};
use serde_json::Value;
use uuid::Uuid;

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

    async fn discover_from_path(
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

        insert_override(
            &mut overrides,
            "session_id",
            resolve_context_value(
                request_meta,
                context_meta,
                &["session_id", "sessionId", "x-session-id", "x_session_id"],
            ),
        );
        insert_override(
            &mut overrides,
            schema::PARENT_SESSION_ID,
            resolve_context_value(
                request_meta,
                context_meta,
                &[
                    schema::PARENT_SESSION_ID,
                    "parentSessionId",
                    "x-parent-session-id",
                    "x_parent_session_id",
                ],
            ),
        );
        insert_override(
            &mut overrides,
            schema::PROJECT_ID,
            resolve_context_value(
                request_meta,
                context_meta,
                &[
                    schema::PROJECT_ID,
                    "projectId",
                    "x-project-id",
                    "x_project_id",
                ],
            ),
        );
        insert_override(
            &mut overrides,
            schema::WORKTREE_ID,
            resolve_context_value(
                request_meta,
                context_meta,
                &[
                    schema::WORKTREE_ID,
                    "worktreeId",
                    "x-worktree-id",
                    "x_worktree_id",
                ],
            ),
        );
        insert_override(
            &mut overrides,
            schema::REPO_ID,
            resolve_context_value(
                request_meta,
                context_meta,
                &[schema::REPO_ID, "repoId", "x-repo-id", "x_repo_id"],
            ),
        );
        insert_override(
            &mut overrides,
            schema::REPO_PATH,
            resolve_context_value(
                request_meta,
                context_meta,
                &[schema::REPO_PATH, "repoPath", "x-repo-path", "x_repo_path"],
            ),
        );
        insert_override(
            &mut overrides,
            "workspace_root",
            resolve_context_value(
                request_meta,
                context_meta,
                &["workspace_root", "workspaceRoot", "x-workspace-root"],
            ),
        );
        insert_override(
            &mut overrides,
            "operator_id",
            resolve_context_value(
                request_meta,
                context_meta,
                &[
                    "operator_id",
                    "operatorId",
                    "x-operator-id",
                    "x_operator_id",
                ],
            ),
        );
        insert_override(
            &mut overrides,
            "machine_id",
            resolve_context_value(
                request_meta,
                context_meta,
                &["machine_id", "machineId", "x-machine-id", "x_machine_id"],
            ),
        );
        insert_override(
            &mut overrides,
            "agent_program",
            resolve_context_value(
                request_meta,
                context_meta,
                &[
                    "agent_program",
                    "agentProgram",
                    "ide",
                    "x-agent-program",
                    "x_agent_program",
                ],
            ),
        );
        insert_override(
            &mut overrides,
            "model_id",
            resolve_context_value(
                request_meta,
                context_meta,
                &["model_id", "model", "modelId", "x-model-id", "x_model_id"],
            ),
        );
        if let Some(delegated) = resolve_context_bool(
            request_meta,
            context_meta,
            &["delegated", "is_delegated", "isDelegated", "x-delegated"],
        ) {
            overrides.insert("delegated".to_owned(), delegated.to_string());
        }
        insert_override(
            &mut overrides,
            "execution_flow",
            resolve_context_value(
                request_meta,
                context_meta,
                &[
                    "execution_flow",
                    "executionFlow",
                    "x-execution-flow",
                    "x_execution_flow",
                ],
            ),
        );

        overrides
    }

    /// Resolve execution context from explicit overrides and runtime defaults.
    #[must_use]
    pub fn resolve(defaults: &RuntimeDefaults, overrides: &HashMap<String, String>) -> Self {
        let session_id = resolve_override_value(
            overrides,
            &["session_id", "sessionId", "x-session-id", "x_session_id"],
        )
        .or_else(|| defaults.session_id.clone());
        let parent_session_id = resolve_override_value(
            overrides,
            &[
                schema::PARENT_SESSION_ID,
                "parentSessionId",
                "x-parent-session-id",
                "x_parent_session_id",
            ],
        );
        let project_id = resolve_override_value(
            overrides,
            &[
                schema::PROJECT_ID,
                "projectId",
                "x-project-id",
                "x_project_id",
            ],
        );
        let worktree_id = resolve_override_value(
            overrides,
            &[
                schema::WORKTREE_ID,
                "worktreeId",
                "x-worktree-id",
                "x_worktree_id",
            ],
        );
        let repo_id = resolve_override_value(
            overrides,
            &[schema::REPO_ID, "repoId", "x-repo-id", "x_repo_id"],
        )
        .or_else(|| defaults.repo_id.clone());
        let repo_path = resolve_override_value(
            overrides,
            &[schema::REPO_PATH, "repoPath", "x-repo-path", "x_repo_path"],
        )
        .or_else(|| {
            resolve_override_value(
                overrides,
                &["workspace_root", "workspaceRoot", "x-workspace-root"],
            )
        })
        .or_else(|| defaults.repo_path.clone())
        .or_else(|| defaults.workspace_root.clone());
        let operator_id = resolve_override_value(
            overrides,
            &[
                "operator_id",
                "operatorId",
                "x-operator-id",
                "x_operator_id",
            ],
        )
        .or_else(|| defaults.operator_id.clone());
        let machine_id = resolve_override_value(
            overrides,
            &["machine_id", "machineId", "x-machine-id", "x_machine_id"],
        )
        .or_else(|| defaults.machine_id.clone());
        let agent_program = resolve_override_value(
            overrides,
            &[
                "agent_program",
                "agentProgram",
                "ide",
                "x-agent-program",
                "x_agent_program",
            ],
        )
        .or_else(|| defaults.agent_program.clone());
        let model_id = resolve_override_value(
            overrides,
            &["model_id", "model", "modelId", "x-model-id", "x_model_id"],
        )
        .or_else(|| defaults.model_id.clone());
        let delegated = resolve_override_bool(
            overrides,
            &["delegated", "is_delegated", "isDelegated", "x-delegated"],
        )
        .or(Some(parent_session_id.is_some()));
        let execution_flow = resolve_override_value(
            overrides,
            &[
                "execution_flow",
                "executionFlow",
                "x-execution-flow",
                "x_execution_flow",
            ],
        )
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
        insert_argument_if_missing(request, "session_id", self.session_id.as_deref());
        insert_argument_if_missing(
            request,
            "parent_session_id",
            self.parent_session_id.as_deref(),
        );
        insert_argument_if_missing(request, "project_id", self.project_id.as_deref());
        insert_argument_if_missing(request, "worktree_id", self.worktree_id.as_deref());
        insert_argument_if_missing(request, "repo_id", self.repo_id.as_deref());
        insert_argument_if_missing(request, "repo_path", self.repo_path.as_deref());
        insert_argument_if_missing(request, "operator_id", self.operator_id.as_deref());
        insert_argument_if_missing(request, "machine_id", self.machine_id.as_deref());
        insert_argument_if_missing(request, "agent_program", self.agent_program.as_deref());
        insert_argument_if_missing(request, "model_id", self.model_id.as_deref());
        insert_bool_argument_if_missing(request, "delegated", self.delegated);
        insert_i64_argument_if_missing(request, "timestamp", self.timestamp);
        insert_argument_if_missing(request, "execution_flow", self.execution_flow.as_deref());
    }
}

fn insert_i64_argument_if_missing(
    request: &mut CallToolRequestParams,
    key: &'static str,
    value: Option<i64>,
) {
    let Some(value) = value else {
        return;
    };

    let arguments = request.arguments.get_or_insert_with(Default::default);
    arguments
        .entry(key.to_owned())
        .or_insert_with(|| Value::Number(serde_json::Number::from(value)));
}

fn insert_bool_argument_if_missing(
    request: &mut CallToolRequestParams,
    key: &'static str,
    value: Option<bool>,
) {
    let Some(value) = value else {
        return;
    };

    let arguments = request.arguments.get_or_insert_with(Default::default);
    arguments
        .entry(key.to_owned())
        .or_insert_with(|| Value::Bool(value));
}

fn insert_argument_if_missing(
    request: &mut CallToolRequestParams,
    key: &'static str,
    value: Option<&str>,
) {
    let Some(value) = value else {
        return;
    };

    let arguments = request.arguments.get_or_insert_with(Default::default);
    arguments
        .entry(key.to_owned())
        .or_insert_with(|| Value::String(value.to_owned()));
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

fn validate_execution_context(
    tool_name: &str,
    execution_context: &ToolExecutionContext,
) -> Result<(), McpError> {
    validate_operation_mode_matrix(tool_name, execution_context)?;

    let requires_provenance = matches!(tool_name, "index" | "search" | "memory");
    if !requires_provenance {
        return Ok(());
    }

    let mut missing = Vec::new();
    if is_missing_text(&execution_context.session_id) {
        missing.push("session_id");
    }
    if is_missing_text(&execution_context.repo_id) {
        missing.push("repo_id");
    }
    if is_missing_text(&execution_context.repo_path) {
        missing.push("repo_path");
    }
    if is_missing_text(&execution_context.operator_id) {
        missing.push("operator_id");
    }
    if is_missing_text(&execution_context.machine_id) {
        missing.push("machine_id");
    }
    if is_missing_text(&execution_context.agent_program) {
        missing.push("agent_program");
    }
    if is_missing_text(&execution_context.model_id) {
        missing.push("model_id");
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
        &[ExecutionFlow::StdioOnly, ExecutionFlow::ClientHybrid, ExecutionFlow::ServerHybrid]
    };

    if allowed.contains(&flow) {
        Ok(())
    } else {
        Err(McpError::invalid_params(
            format!(
                "Operation mode matrix violation for '{tool_name}': flow '{}' is not allowed. Allowed flows: {}",
                flow,
                allowed.iter().map(|f| f.as_str()).collect::<Vec<_>>().join(", ")
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
    if let Some(parent_session_id) = &execution_context.parent_session_id {
        context = context.with_metadata("parent_session_id", parent_session_id.as_str());
    }
    if let Some(project_id) = &execution_context.project_id {
        context = context.with_metadata("project_id", project_id.as_str());
    }
    if let Some(worktree_id) = &execution_context.worktree_id {
        context = context.with_metadata("worktree_id", worktree_id.as_str());
    }
    if let Some(repo_id) = &execution_context.repo_id {
        context = context.with_metadata("repo_id", repo_id.as_str());
    }
    if let Some(repo_path) = &execution_context.repo_path {
        context = context.with_metadata("repo_path", repo_path.as_str());
    }
    if let Some(operator_id) = &execution_context.operator_id {
        context = context.with_metadata("operator_id", operator_id.as_str());
    }
    if let Some(machine_id) = &execution_context.machine_id {
        context = context.with_metadata("machine_id", machine_id.as_str());
    }
    if let Some(agent_program) = &execution_context.agent_program {
        context = context.with_metadata("agent_program", agent_program.as_str());
    }
    if let Some(model_id) = &execution_context.model_id {
        context = context.with_metadata("model_id", model_id.as_str());
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use async_trait::async_trait;
    use mcb_domain::entities::vcs::{RefDiff, VcsBranch, VcsCommit, VcsRepository};
    use mcb_domain::error::{Error, Result};
    use mcb_domain::value_objects::RepositoryId;

    use super::{ExecutionFlow, RuntimeDefaults, ToolExecutionContext, validate_execution_context};

    struct TestVcsProvider {
        repo_root: PathBuf,
        repo_id: RepositoryId,
    }

    #[async_trait]
    impl mcb_domain::ports::VcsProvider for TestVcsProvider {
        async fn open_repository(&self, path: &Path) -> Result<VcsRepository> {
            if path.starts_with(&self.repo_root) {
                return Ok(VcsRepository::new(
                    self.repo_id,
                    path.to_path_buf(),
                    "main".to_owned(),
                    vec!["main".to_owned()],
                    None,
                ));
            }

            Err(Error::vcs("not a repository"))
        }

        fn repository_id(&self, _repo: &VcsRepository) -> RepositoryId {
            self.repo_id
        }

        async fn list_branches(&self, _repo: &VcsRepository) -> Result<Vec<VcsBranch>> {
            Ok(Vec::new())
        }

        async fn commit_history(
            &self,
            _repo: &VcsRepository,
            _branch: &str,
            _limit: Option<usize>,
        ) -> Result<Vec<VcsCommit>> {
            Ok(Vec::new())
        }

        async fn list_files(&self, _repo: &VcsRepository, _branch: &str) -> Result<Vec<PathBuf>> {
            Ok(Vec::new())
        }

        async fn read_file(
            &self,
            _repo: &VcsRepository,
            _branch: &str,
            _path: &Path,
        ) -> Result<String> {
            Ok(String::new())
        }

        fn vcs_name(&self) -> &str {
            "test"
        }

        async fn diff_refs(
            &self,
            _repo: &VcsRepository,
            _base_ref: &str,
            _head_ref: &str,
        ) -> Result<RefDiff> {
            Err(Error::vcs("not implemented"))
        }

        async fn list_repositories(&self, _root: &Path) -> Result<Vec<VcsRepository>> {
            Ok(Vec::new())
        }
    }

    fn valid_context() -> ToolExecutionContext {
        ToolExecutionContext {
            session_id: Some("session-1".to_owned()),
            parent_session_id: Some("parent-1".to_owned()),
            project_id: Some("project-1".to_owned()),
            worktree_id: Some("wt-1".to_owned()),
            repo_id: Some("repo-1".to_owned()),
            repo_path: Some("/tmp/repo".to_owned()),
            operator_id: Some("operator-1".to_owned()),
            machine_id: Some("machine-1".to_owned()),
            agent_program: Some("opencode".to_owned()),
            model_id: Some("gpt-5.3-codex".to_owned()),
            delegated: Some(false),
            timestamp: Some(1),
            execution_flow: Some(ExecutionFlow::StdioOnly.to_string()),
        }
    }

    #[test]
    fn rejects_blank_provenance_scope_for_search() {
        let mut context = valid_context();
        context.operator_id = Some("   ".to_owned());

        let validation = validate_execution_context("search", &context);
        assert!(validation.is_err(), "blank operator_id must be rejected");
        let error = match validation {
            Ok(()) => return,
            Err(error) => error,
        };
        assert!(error.message.contains("operator_id"));
    }

    #[test]
    fn rejects_delegated_without_parent_session_id() {
        let mut context = valid_context();
        context.delegated = Some(true);
        context.parent_session_id = Some(" ".to_owned());

        let validation = validate_execution_context("memory", &context);
        assert!(
            validation.is_err(),
            "delegated context must include parent_session_id"
        );
        let error = match validation {
            Ok(()) => return,
            Err(error) => error,
        };
        assert!(error.message.contains("parent_session_id"));
    }

    #[test]
    fn non_provenance_tool_bypasses_scope_gate() {
        let context = ToolExecutionContext {
            session_id: None,
            parent_session_id: None,
            project_id: None,
            worktree_id: None,
            repo_id: None,
            repo_path: None,
            operator_id: None,
            machine_id: None,
            agent_program: None,
            model_id: None,
            delegated: None,
            timestamp: None,
            execution_flow: Some(ExecutionFlow::StdioOnly.to_string()),
        };

        assert!(
            validate_execution_context("validate", &context).is_ok(),
            "non-index/search/memory tools should not require provenance scope"
        );
    }

    #[test]
    fn rejects_validate_in_server_hybrid_flow() {
        let mut context = valid_context();
        context.execution_flow = Some(ExecutionFlow::ServerHybrid.to_string());

        let validation = validate_execution_context("validate", &context);
        assert!(
            validation.is_err(),
            "validate must be rejected in server-hybrid flow"
        );
        let err = match validation {
            Ok(()) => return,
            Err(error) => error,
        };
        assert!(err.message.contains("Operation mode matrix violation"));
    }

    #[test]
    fn allows_search_in_client_hybrid_flow() {
        let mut context = valid_context();
        context.execution_flow = Some(ExecutionFlow::ClientHybrid.to_string());

        let validation = validate_execution_context("search", &context);
        assert!(
            validation.is_ok(),
            "search must be allowed in client-hybrid flow"
        );
    }

    #[test]
    fn allows_search_in_server_hybrid_flow() {
        let mut context = valid_context();
        context.execution_flow = Some(ExecutionFlow::ServerHybrid.to_string());

        assert!(
            validate_execution_context("search", &context).is_ok(),
            "search must be allowed in server-hybrid flow"
        );
    }

    #[tokio::test]
    async fn test_runtime_defaults_discover() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let repo_root = temp_dir.path().join("repo");
        let nested = repo_root.join("nested").join("path");
        std::fs::create_dir_all(&nested).expect("nested path");

        let provider = TestVcsProvider {
            repo_root: repo_root.clone(),
            repo_id: RepositoryId::from_name("repo-test"),
        };

        let defaults = RuntimeDefaults::discover_from_path(
            &provider,
            Some(nested.as_path()),
            Some(ExecutionFlow::StdioOnly),
        )
        .await;

        assert_eq!(defaults.workspace_root.as_deref(), repo_root.to_str());
        assert_eq!(defaults.repo_path.as_deref(), repo_root.to_str());
        assert_eq!(
            defaults.repo_id.as_deref(),
            Some(RepositoryId::from_name("repo-test").as_str().as_str())
        );
        assert_eq!(defaults.agent_program.as_deref(), Some("mcb-stdio"));
        assert_eq!(defaults.model_id.as_deref(), Some("unknown"));
        assert_eq!(defaults.execution_flow, Some(ExecutionFlow::StdioOnly));
        assert!(defaults.session_id.is_some());
    }

    #[test]
    fn test_resolve_overrides_beat_defaults() {
        let defaults = RuntimeDefaults {
            workspace_root: Some("/defaults/workspace".to_owned()),
            repo_path: Some("/defaults/repo".to_owned()),
            repo_id: Some("repo-default".to_owned()),
            operator_id: Some("operator-default".to_owned()),
            machine_id: Some("machine-default".to_owned()),
            session_id: Some("session-default".to_owned()),
            agent_program: Some("mcb-stdio".to_owned()),
            model_id: Some("unknown".to_owned()),
            execution_flow: Some(ExecutionFlow::StdioOnly),
        };

        let overrides = HashMap::from([
            ("session_id".to_owned(), "session-override".to_owned()),
            ("repo_id".to_owned(), "repo-override".to_owned()),
            ("repo_path".to_owned(), "/repo/override".to_owned()),
            ("operator_id".to_owned(), "operator-override".to_owned()),
            ("machine_id".to_owned(), "machine-override".to_owned()),
            ("agent_program".to_owned(), "agent-override".to_owned()),
            ("model_id".to_owned(), "model-override".to_owned()),
            ("execution_flow".to_owned(), "client-hybrid".to_owned()),
            ("delegated".to_owned(), "true".to_owned()),
        ]);

        let context = ToolExecutionContext::resolve(&defaults, &overrides);

        assert_eq!(context.session_id.as_deref(), Some("session-override"));
        assert_eq!(context.repo_id.as_deref(), Some("repo-override"));
        assert_eq!(context.repo_path.as_deref(), Some("/repo/override"));
        assert_eq!(context.operator_id.as_deref(), Some("operator-override"));
        assert_eq!(context.machine_id.as_deref(), Some("machine-override"));
        assert_eq!(context.agent_program.as_deref(), Some("agent-override"));
        assert_eq!(context.model_id.as_deref(), Some("model-override"));
        assert_eq!(context.execution_flow.as_deref(), Some("client-hybrid"));
        assert_eq!(context.delegated, Some(true));
        assert!(context.timestamp.is_some());
    }

    #[test]
    fn test_resolve_with_empty_overrides_uses_defaults() {
        let defaults = RuntimeDefaults {
            workspace_root: Some("/defaults/workspace".to_owned()),
            repo_path: Some("/defaults/repo".to_owned()),
            repo_id: Some("repo-default".to_owned()),
            operator_id: Some("operator-default".to_owned()),
            machine_id: Some("machine-default".to_owned()),
            session_id: Some("session-default".to_owned()),
            agent_program: Some("mcb-stdio".to_owned()),
            model_id: Some("unknown".to_owned()),
            execution_flow: Some(ExecutionFlow::StdioOnly),
        };

        let context = ToolExecutionContext::resolve(&defaults, &HashMap::new());

        assert_eq!(context.session_id.as_deref(), Some("session-default"));
        assert_eq!(context.repo_id.as_deref(), Some("repo-default"));
        assert_eq!(context.repo_path.as_deref(), Some("/defaults/repo"));
        assert_eq!(context.operator_id.as_deref(), Some("operator-default"));
        assert_eq!(context.machine_id.as_deref(), Some("machine-default"));
        assert_eq!(context.agent_program.as_deref(), Some("mcb-stdio"));
        assert_eq!(context.model_id.as_deref(), Some("unknown"));
        assert_eq!(context.execution_flow.as_deref(), Some("stdio-only"));
    }

    #[test]
    fn test_resolve_workspace_root_maps_to_repo_path() {
        let defaults = RuntimeDefaults {
            workspace_root: None,
            repo_path: None,
            repo_id: None,
            operator_id: None,
            machine_id: None,
            session_id: None,
            agent_program: None,
            model_id: None,
            execution_flow: None,
        };
        let overrides = HashMap::from([(
            "workspace_root".to_owned(),
            "/workspace/override".to_owned(),
        )]);

        let context = ToolExecutionContext::resolve(&defaults, &overrides);
        assert_eq!(context.repo_path.as_deref(), Some("/workspace/override"));
    }
}
