//! Execution context validation and hook processing.
//!
//! Provides validation of execution context against operation mode matrix
//! and provenance scope requirements, plus post-tool-use hook triggering.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use mcb_domain::value_objects::ids::SessionId;

use crate::hooks::{HookProcessor, PostToolUseContext};
use crate::tools::context::ToolExecutionContext;
use crate::tools::defaults::ExecutionFlow;

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

/// Check if a text value is missing or empty.
fn is_missing_text(value: &Option<String>) -> bool {
    value.as_deref().is_none_or(|s| s.trim().is_empty())
}

/// Validate operation mode matrix for tool execution.
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

/// Normalize execution flow string to enum.
fn normalize_execution_flow(flow: Option<&str>) -> Result<ExecutionFlow, McpError> {
    let raw = flow.unwrap_or(ExecutionFlow::StdioOnly.as_str());
    raw.parse::<ExecutionFlow>()
        .map_err(|e| McpError::invalid_params(e, None))
}

/// Trigger post-tool-use hook for memory operations.
///
/// # Errors
///
/// Returns an error string if the hook processor fails.
pub async fn trigger_post_tool_use_hook(
    tool_name: &str,
    result: &CallToolResult,
    hook_processor: &HookProcessor,
    execution_context: &ToolExecutionContext,
) -> Result<(), String> {
    let mut context =
        PostToolUseContext::new(tool_name.to_owned(), result.is_error.unwrap_or(false))
            .map_err(|e| e.to_string())?;

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
