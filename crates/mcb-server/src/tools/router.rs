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
    /// Execution timestamp (Unix timestamp in seconds).
    pub timestamp: Option<i64>,
    /// Optional execution flow identifier for tracing.
    pub execution_flow: Option<String>,
}

impl ToolExecutionContext {
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
    value: Option<&str>,
) {
    let Some(value) = value else {
        return;
    };

    let arguments = request.arguments.get_or_insert_with(Default::default);
    arguments
        .entry(key.to_string())
        .or_insert_with(|| Value::String(value.to_owned()));
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

    let result = dispatch_tool_call(&request, handlers).await?;

    if let Err(e) = trigger_post_tool_use_hook(
        request.name.as_ref(),
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
    validate_operation_mode_matrix(tool_name, execution_context)?;

    let requires_provenance = matches!(tool_name, "index" | "search" | "memory");
    if !requires_provenance {
        return Ok(());
    }

    let mut missing = Vec::new();
    if is_missing_text(&execution_context.session_id) {
        missing.push("session_id");
    }
    if is_missing_text(&execution_context.project_id) {
        missing.push("project_id");
    }
    if is_missing_text(&execution_context.repo_id) {
        missing.push("repo_id");
    }
    if is_missing_text(&execution_context.repo_path) {
        missing.push("repo_path");
    }
    if is_missing_text(&execution_context.worktree_id) {
        missing.push("worktree_id");
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

    let allowed = if matches!(
        tool_name,
        "search"
            | "memory"
            | "session"
            | "agent"
            | "project"
            | "vcs"
            | "vcs_entity"
            | "plan_entity"
            | "issue_entity"
            | "org_entity"
            | "entity"
    ) {
        &["stdio-only", "server-hybrid"][..]
    } else if matches!(tool_name, "validate") {
        &["stdio-only", "client-hybrid"][..]
    } else {
        &["stdio-only", "client-hybrid", "server-hybrid"][..]
    };

    if allowed.contains(&flow) {
        Ok(())
    } else {
        Err(McpError::invalid_params(
            format!(
                "Operation mode matrix violation for '{tool_name}': flow '{flow}' is not allowed. Allowed flows: {}",
                allowed.join(", ")
            ),
            None,
        ))
    }
}

fn normalize_execution_flow(flow: Option<&str>) -> Result<&'static str, McpError> {
    let normalized = flow.unwrap_or("stdio-only").trim().to_ascii_lowercase();
    match normalized.as_str() {
        "stdio-only" => Ok("stdio-only"),
        "client-hybrid" => Ok("client-hybrid"),
        "server-hybrid" => Ok("server-hybrid"),
        _ => Err(McpError::invalid_params(
            format!(
                "Invalid execution_flow '{normalized}'. Expected one of: stdio-only, client-hybrid, server-hybrid"
            ),
            None,
        )),
    }
}

async fn trigger_post_tool_use_hook(
    tool_name: &str,
    result: &CallToolResult,
    hook_processor: &HookProcessor,
    execution_context: &ToolExecutionContext,
) -> Result<(), String> {
    let mut context =
        PostToolUseContext::new(tool_name.to_string(), result.is_error.unwrap_or(false));

    if let Some(session_id) = &execution_context.session_id {
        context = context.with_session_id(SessionId::new(session_id));
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
    use super::{ToolExecutionContext, validate_execution_context};

    fn valid_context() -> ToolExecutionContext {
        ToolExecutionContext {
            session_id: Some("session-1".to_string()),
            parent_session_id: Some("parent-1".to_string()),
            project_id: Some("project-1".to_string()),
            worktree_id: Some("wt-1".to_string()),
            repo_id: Some("repo-1".to_string()),
            repo_path: Some("/tmp/repo".to_string()),
            operator_id: Some("operator-1".to_string()),
            machine_id: Some("machine-1".to_string()),
            agent_program: Some("opencode".to_string()),
            model_id: Some("gpt-5.3-codex".to_string()),
            delegated: Some(false),
            timestamp: Some(1),
            execution_flow: Some("stdio-only".to_string()),
        }
    }

    #[test]
    fn rejects_blank_provenance_scope_for_search() {
        let mut context = valid_context();
        context.project_id = Some("   ".to_string());

        let error = validate_execution_context("search", &context)
            .expect_err("blank project_id must be rejected");
        assert!(error.message.contains("project_id"));
    }

    #[test]
    fn rejects_delegated_without_parent_session_id() {
        let mut context = valid_context();
        context.delegated = Some(true);
        context.parent_session_id = Some(" ".to_string());

        let error = validate_execution_context("memory", &context)
            .expect_err("delegated context must include parent_session_id");
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
            execution_flow: Some("stdio-only".to_string()),
        };

        validate_execution_context("validate", &context)
            .expect("non-index/search/memory tools should not require provenance scope");
    }

    #[test]
    fn rejects_validate_in_server_hybrid_flow() {
        let mut context = valid_context();
        context.execution_flow = Some("server-hybrid".to_string());

        let err = validate_execution_context("validate", &context)
            .expect_err("validate must be rejected in server-hybrid flow");
        assert!(err.message.contains("Operation mode matrix violation"));
    }

    #[test]
    fn rejects_search_in_client_hybrid_flow() {
        let mut context = valid_context();
        context.execution_flow = Some("client-hybrid".to_string());

        let err = validate_execution_context("search", &context)
            .expect_err("search must be rejected in client-hybrid flow");
        assert!(err.message.contains("Operation mode matrix violation"));
    }

    #[test]
    fn allows_search_in_server_hybrid_flow() {
        let mut context = valid_context();
        context.execution_flow = Some("server-hybrid".to_string());

        validate_execution_context("search", &context)
            .expect("search must be allowed in server-hybrid flow");
    }
}
