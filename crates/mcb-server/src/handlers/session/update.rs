use std::sync::Arc;

use mcb_domain::constants::keys as schema;
use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus};
use mcb_domain::ports::services::AgentSessionServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::Map;
use serde_json::Value;

use super::common::{json_map, opt_str, require_session_id_str};
use crate::args::SessionArgs;
use crate::constants::fields::FIELD_UPDATED;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::utils::mcp::{resolve_identifier_precedence, tool_error};
use tracing::error;

/// Updates an existing agent session.
#[tracing::instrument(skip_all)]
pub async fn update_session(
    agent_service: &Arc<dyn AgentSessionServiceInterface>,
    args: &SessionArgs,
) -> Result<CallToolResult, McpError> {
    let session_id = match require_session_id_str(args) {
        Ok(id) => id,
        Err(error_result) => return Ok(error_result),
    };
    let data = json_map(&args.data);
    let status = parse_status(args, data)?;
    match agent_service.get_session(&session_id).await {
        Ok(Some(mut session)) => {
            apply_resolved_identifier(
                &mut session.project_id,
                schema::PROJECT_ID,
                args.project_id.as_deref(),
                payload_str(data, schema::PROJECT_ID).as_deref(),
            )?;
            apply_resolved_identifier(
                &mut session.worktree_id,
                schema::WORKTREE_ID,
                args.worktree_id.as_deref(),
                payload_str(data, schema::WORKTREE_ID).as_deref(),
            )?;

            if let Some(status) = status {
                session.status = status;
            }
            if let Some(data) = data {
                apply_session_updates(&mut session, data);
            }
            let status = session.status.as_str().to_owned();
            match agent_service.update_session(session).await {
                Ok(_) => ResponseFormatter::json_success(&serde_json::json!({
                    schema::ID: session_id,
                    schema::STATUS: status,
                    (FIELD_UPDATED): true,
                })),
                Err(e) => {
                    error!("Failed to update agent session: {:?}", e);
                    Ok(to_contextual_tool_error(e))
                }
            }
        }
        Ok(None) => Ok(tool_error("Agent session not found")),
        Err(e) => {
            error!("Failed to update agent session (get failed): {:?}", e);
            Ok(to_contextual_tool_error(e))
        }
    }
}

fn parse_status(
    args: &SessionArgs,
    data: Option<&Map<String, Value>>,
) -> Result<Option<AgentSessionStatus>, McpError> {
    let status_value = args.status.clone().or_else(|| {
        data.and_then(|d| {
            d.get(schema::STATUS)
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
    });

    status_value
        .map(|status| {
            status
                .parse()
                .map_err(|e: String| McpError::invalid_params(e, None))
        })
        .transpose()
}

fn payload_str(data: Option<&Map<String, Value>>, key: &str) -> Option<String> {
    data.and_then(|d| opt_str(d, key))
}

fn apply_resolved_identifier(
    session_value: &mut Option<String>,
    field_name: &str,
    args_value: Option<&str>,
    payload_value: Option<&str>,
) -> Result<(), McpError> {
    let resolved = resolve_identifier_precedence(field_name, args_value, payload_value)?;
    if let Some(value) = resolved {
        if let Some(existing) = session_value.as_deref()
            && existing != value
        {
            return Err(McpError::invalid_params(
                format!("conflicting {field_name}: args/data='{value}', session='{existing}'"),
                None,
            ));
        }
        *session_value = Some(value);
    }
    Ok(())
}

fn apply_session_updates(session: &mut AgentSession, data: &Map<String, Value>) {
    session.result_summary = data
        .get(schema::RESULT_SUMMARY)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or(session.result_summary.clone());
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
