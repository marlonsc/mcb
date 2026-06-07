//! Tool call routing and handler dispatch.
//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Routes incoming tool call requests to the appropriate handlers.
//! This module provides a centralized dispatch mechanism for MCP tool calls.

use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolRequestParams, CallToolResult};

use crate::handlers::{
    AgentHandler, EntityHandler, IndexHandler, IssueEntityHandler, MemoryHandler, OrgEntityHandler,
    PlanEntityHandler, ProjectHandler, SearchHandler, SessionHandler, ValidateHandler,
    VcsEntityHandler, VcsHandler,
};
use crate::hooks::HookProcessor;
use crate::tools::context::ToolExecutionContext;
use crate::tools::dispatch_tool_call;
use crate::tools::validation::{trigger_post_tool_use_hook, validate_execution_context};
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
        mcb_domain::warn!("ToolRouter", "PostToolUse hook failed (non-fatal)", &e);
    }

    Ok(result)
}
