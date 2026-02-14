use std::sync::Arc;

use mcb_domain::constants::keys as schema;
use mcb_domain::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::Value;

use crate::args::SessionArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handler_helpers::resolve_identifier_precedence;
use crate::utils::json;
use tracing::error;

/// Updates an existing agent session.
#[tracing::instrument(skip_all)]
pub async fn update_session(
    agent_service: &Arc<dyn AgentSessionServiceInterface>,
    args: &SessionArgs,
) -> Result<CallToolResult, McpError> {
    let session_id = match args.session_id.as_ref() {
        Some(id) => id.as_str(),
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing session_id",
            )]));
        }
    };
    let data = json::json_map(&args.data);
    let status = match args.status.as_ref() {
        Some(status) => Some(
            status
                .parse()
                .map_err(|e: String| McpError::invalid_params(e, None))?,
        ),
        None => data
            .and_then(|d| {
                d.get(schema::STATUS)
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .map(|status| {
                status
                    .parse()
                    .map_err(|e: String| McpError::invalid_params(e, None))
            })
            .transpose()?,
    };
    match agent_service.get_session(session_id).await {
        Ok(Some(mut session)) => {
            let payload_project_id = data.and_then(|d| {
                d.get(schema::PROJECT_ID)
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            });
            let payload_worktree_id = data.and_then(|d| {
                d.get(schema::WORKTREE_ID)
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            });

            let resolved_project_id = resolve_identifier_precedence(
                schema::PROJECT_ID,
                args.project_id.as_deref(),
                payload_project_id.as_deref(),
            )?;
            if let Some(project_id) = resolved_project_id {
                if let Some(existing) = session.project_id.as_deref()
                    && existing != project_id
                {
                    return Err(McpError::invalid_params(
                        format!(
                            "conflicting project_id: args/data='{project_id}', session='{existing}'"
                        ),
                        None,
                    ));
                }
                session.project_id = Some(project_id);
            }

            let resolved_worktree_id = resolve_identifier_precedence(
                schema::WORKTREE_ID,
                args.worktree_id.as_deref(),
                payload_worktree_id.as_deref(),
            )?;
            if let Some(worktree_id) = resolved_worktree_id {
                if let Some(existing) = session.worktree_id.as_deref()
                    && existing != worktree_id
                {
                    return Err(McpError::invalid_params(
                        format!(
                            "conflicting worktree_id: args/data='{worktree_id}', session='{existing}'"
                        ),
                        None,
                    ));
                }
                session.worktree_id = Some(worktree_id);
            }

            if let Some(status) = status {
                session.status = status;
            }
            if let Some(data) = data {
                session.result_summary = data
                    .get(schema::RESULT_SUMMARY)
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .or(session.result_summary);
                session.token_count = data
                    .get(schema::TOKEN_COUNT)
                    .and_then(Value::as_i64)
                    .or(session.token_count);
                session.tool_calls_count = data
                    .get(schema::TOOL_CALLS_COUNT)
                    .and_then(Value::as_i64)
                    .or(session.tool_calls_count);
                session.delegations_count = data
                    .get(schema::DELEGATIONS_COUNT)
                    .and_then(Value::as_i64)
                    .or(session.delegations_count);
            }
            let status = session.status.as_str().to_owned();
            match agent_service.update_session(session).await {
                Ok(_) => ResponseFormatter::json_success(&serde_json::json!({
                    schema::ID: session_id,
                    schema::STATUS: status,
                    "updated": true,
                })),
                Err(e) => {
                    error!("Failed to update agent session: {:?}", e);
                    Ok(to_contextual_tool_error(e))
                }
            }
        }
        Ok(None) => Ok(CallToolResult::error(vec![Content::text(
            "Agent session not found",
        )])),
        Err(e) => {
            error!("Failed to update agent session (get failed): {:?}", e);
            Ok(to_contextual_tool_error(e))
        }
    }
}
