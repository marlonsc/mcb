//!
//! Routes incoming tool call requests to the appropriate handlers.
//! This module provides a centralized dispatch mechanism for MCP tool calls.

use std::sync::Arc;

use mcb_domain::value_objects::ids::SessionId;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use serde_json::Value;
use tracing::warn;

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
    pub timestamp: Option<i64>,
}

impl ToolExecutionContext {
    /// Inject execution context into tool arguments when those keys are missing.
    pub fn apply_to_request_if_missing(&self, request: &mut CallToolRequestParams) {
        insert_argument_if_missing(request, "session_id", self.session_id.clone());
        insert_argument_if_missing(request, "parent_session_id", self.parent_session_id.clone());
        insert_argument_if_missing(request, "project_id", self.project_id.clone());
        insert_argument_if_missing(request, "worktree_id", self.worktree_id.clone());
        insert_argument_if_missing(request, "repo_id", self.repo_id.clone());
        insert_argument_if_missing(request, "repo_path", self.repo_path.clone());
        insert_argument_if_missing(request, "operator_id", self.operator_id.clone());
        insert_argument_if_missing(request, "machine_id", self.machine_id.clone());
        insert_argument_if_missing(request, "agent_program", self.agent_program.clone());
        insert_argument_if_missing(request, "model_id", self.model_id.clone());
        insert_bool_argument_if_missing(request, "delegated", self.delegated);
        insert_i64_argument_if_missing(request, "timestamp", self.timestamp);
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
        .entry(key.to_string())
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
        .entry(key.to_string())
        .or_insert_with(|| Value::Bool(value));
}

fn insert_argument_if_missing(
    request: &mut CallToolRequestParams,
    key: &'static str,
    value: Option<String>,
) {
    let Some(value) = value else {
        return;
    };

    let arguments = request.arguments.get_or_insert_with(Default::default);
    arguments
        .entry(key.to_string())
        .or_insert_with(|| Value::String(value));
}

/// Route a tool call request to the appropriate handler
///
/// Parses the request arguments and delegates to the matching handler.
/// After tool execution, automatically triggers PostToolUse hook for memory operations.
pub async fn route_tool_call(
    request: CallToolRequestParams,
    handlers: &ToolHandlers,
    execution_context: ToolExecutionContext,
) -> Result<CallToolResult, McpError> {
    validate_execution_context(request.name.as_ref(), &execution_context)?;

    let tool_name = request.name.clone();
    let result = dispatch_tool_call(&request, handlers).await?;

    if let Err(e) = trigger_post_tool_use_hook(
        &tool_name,
        &result,
        &handlers.hook_processor,
        &execution_context,
    )
    .await
    {
        warn!("PostToolUse hook failed (non-fatal): {}", e);
    }

    Ok(result)
}

fn validate_execution_context(
    tool_name: &str,
    execution_context: &ToolExecutionContext,
) -> Result<(), McpError> {
    let requires_provenance = matches!(tool_name, "index" | "search" | "memory");
    if !requires_provenance {
        return Ok(());
    }

    let mut missing = Vec::new();
    if execution_context.session_id.is_none() {
        missing.push("session_id");
    }
    if execution_context.project_id.is_none() {
        missing.push("project_id");
    }
    if execution_context.repo_id.is_none() {
        missing.push("repo_id");
    }
    if execution_context.repo_path.is_none() {
        missing.push("repo_path");
    }
    if execution_context.worktree_id.is_none() {
        missing.push("worktree_id");
    }
    if execution_context.operator_id.is_none() {
        missing.push("operator_id");
    }
    if execution_context.machine_id.is_none() {
        missing.push("machine_id");
    }
    if execution_context.agent_program.is_none() {
        missing.push("agent_program");
    }
    if execution_context.model_id.is_none() {
        missing.push("model_id");
    }
    if execution_context.delegated.is_none() {
        missing.push("delegated");
    }
    if execution_context.timestamp.is_none() {
        missing.push("timestamp");
    }
    if execution_context.delegated == Some(true) && execution_context.parent_session_id.is_none() {
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

async fn trigger_post_tool_use_hook(
    tool_name: &str,
    result: &CallToolResult,
    hook_processor: &HookProcessor,
    execution_context: &ToolExecutionContext,
) -> Result<(), String> {
    let mut context = PostToolUseContext::new(tool_name.to_string(), result.clone());

    if let Some(session_id) = &execution_context.session_id {
        context = context.with_session_id(SessionId::new(session_id));
    }
    if let Some(parent_session_id) = &execution_context.parent_session_id {
        context = context.with_metadata("parent_session_id".to_string(), parent_session_id.clone());
    }
    if let Some(project_id) = &execution_context.project_id {
        context = context.with_metadata("project_id".to_string(), project_id.clone());
    }
    if let Some(worktree_id) = &execution_context.worktree_id {
        context = context.with_metadata("worktree_id".to_string(), worktree_id.clone());
    }
    if let Some(repo_id) = &execution_context.repo_id {
        context = context.with_metadata("repo_id".to_string(), repo_id.clone());
    }
    if let Some(repo_path) = &execution_context.repo_path {
        context = context.with_metadata("repo_path".to_string(), repo_path.clone());
    }
    if let Some(operator_id) = &execution_context.operator_id {
        context = context.with_metadata("operator_id".to_string(), operator_id.clone());
    }
    if let Some(machine_id) = &execution_context.machine_id {
        context = context.with_metadata("machine_id".to_string(), machine_id.clone());
    }
    if let Some(agent_program) = &execution_context.agent_program {
        context = context.with_metadata("agent_program".to_string(), agent_program.clone());
    }
    if let Some(model_id) = &execution_context.model_id {
        context = context.with_metadata("model_id".to_string(), model_id.clone());
    }
    if let Some(delegated) = execution_context.delegated {
        context = context.with_metadata("delegated".to_string(), delegated.to_string());
    }
    if let Some(timestamp) = execution_context.timestamp {
        context = context.with_metadata("timestamp".to_string(), timestamp.to_string());
    }

    hook_processor
        .process_post_tool_use(context)
        .await
        .map_err(|e| e.to_string())
}
