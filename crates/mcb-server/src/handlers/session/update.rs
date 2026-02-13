use std::sync::Arc;

use mcb_domain::constants::keys as schema;
use mcb_domain::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use super::helpers::SessionHelpers;
use crate::args::SessionArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handler_helpers::resolve_identifier_precedence;
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
    let data = SessionHelpers::json_map(&args.data);
    let status = match args.status.as_ref() {
        Some(status) => Some(SessionHelpers::parse_status(status)?),
        None => data
            .and_then(|d| SessionHelpers::get_str(d, schema::STATUS))
            .map(|status| SessionHelpers::parse_status(&status))
            .transpose()?,
    };
    match agent_service.get_session(session_id).await {
        Ok(Some(mut session)) => {
            let payload_project_id =
                data.and_then(|d| SessionHelpers::get_str(d, schema::PROJECT_ID));
            let payload_worktree_id =
                data.and_then(|d| SessionHelpers::get_str(d, schema::WORKTREE_ID));

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
                session.result_summary = SessionHelpers::get_str(data, schema::RESULT_SUMMARY)
                    .or(session.result_summary);
                session.token_count =
                    SessionHelpers::get_i64(data, schema::TOKEN_COUNT).or(session.token_count);
                session.tool_calls_count = SessionHelpers::get_i64(data, schema::TOOL_CALLS_COUNT)
                    .or(session.tool_calls_count);
                session.delegations_count =
                    SessionHelpers::get_i64(data, schema::DELEGATIONS_COUNT)
                        .or(session.delegations_count);
            }
            let status_str = session.status.as_str().to_string();
            match agent_service.update_session(session).await {
                Ok(_) => ResponseFormatter::json_success(&serde_json::json!({
                    schema::ID: session_id,
                    schema::STATUS: &status_str,
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
