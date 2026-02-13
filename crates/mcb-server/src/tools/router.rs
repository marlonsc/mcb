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
    }
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

    hook_processor
        .process_post_tool_use(context)
        .await
        .map_err(|e| e.to_string())
}
