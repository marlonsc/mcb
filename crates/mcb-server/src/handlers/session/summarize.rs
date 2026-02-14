use std::sync::Arc;

use mcb_domain::ports::services::{CreateSessionSummaryInput, MemoryServiceInterface};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::Value;

use crate::args::SessionArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handler_helpers::{OriginContextInput, resolve_origin_context};
use crate::utils::json;

/// Creates or retrieves a session summary.
#[tracing::instrument(skip_all)]
pub async fn summarize_session(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &SessionArgs,
) -> Result<CallToolResult, McpError> {
    let session_id = match args.session_id.as_ref() {
        Some(id) => id,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing session_id",
            )]));
        }
    };
    if let Some(data) = json::json_map(&args.data) {
        let topics = data
            .get("topics")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        let decisions = data
            .get("decisions")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        let next_steps = data
            .get("next_steps")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        let key_files = data
            .get("key_files")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        let payload_project_id = data
            .get("project_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let payload_session_id = data
            .get("session_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let payload_worktree_id = data
            .get("worktree_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let payload_parent_session_id = data
            .get("parent_session_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let payload_repo_path = data
            .get("repo_path")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let payload_operator_id = data
            .get("operator_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let payload_machine_id = data
            .get("machine_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let payload_agent_program = data
            .get("agent_program")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let payload_model_id = data
            .get("model_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let payload_delegated = data.get("delegated").and_then(Value::as_bool);
        let origin_context = resolve_origin_context(OriginContextInput {
            org_id: args.org_id.as_deref(),
            project_id_args: args.project_id.as_deref(),
            project_id_payload: payload_project_id.as_deref(),
            session_from_args: Some(session_id.as_str()),
            session_from_data: payload_session_id.as_deref(),
            parent_session_from_args: args.parent_session_id.as_deref(),
            parent_session_from_data: payload_parent_session_id.as_deref(),
            execution_from_args: None,
            execution_from_data: None,
            tool_name_args: Some("session"),
            tool_name_payload: None,
            repo_id_args: None,
            repo_id_payload: None,
            repo_path_args: None,
            repo_path_payload: payload_repo_path.as_deref(),
            worktree_id_args: args.worktree_id.as_deref(),
            worktree_id_payload: payload_worktree_id.as_deref(),
            file_path_args: None,
            file_path_payload: None,
            branch_args: None,
            branch_payload: None,
            commit_args: None,
            commit_payload: None,
            operator_id_args: None,
            operator_id_payload: payload_operator_id.as_deref(),
            machine_id_args: None,
            machine_id_payload: payload_machine_id.as_deref(),
            agent_program_args: None,
            agent_program_payload: payload_agent_program.as_deref(),
            model_id_args: None,
            model_id_payload: payload_model_id.as_deref(),
            delegated_args: None,
            delegated_payload: payload_delegated,
            require_project_id: true,
            timestamp: None,
        })?;
        let project_id = origin_context.project_id.clone().ok_or_else(|| {
            McpError::invalid_params("project_id is required for summarize", None)
        })?;
        match memory_service
            .create_session_summary(CreateSessionSummaryInput {
                project_id,
                session_id: session_id.clone(),
                topics,
                decisions,
                next_steps,
                key_files,
                origin_context: Some(origin_context),
            })
            .await
        {
            Ok(summary_id) => ResponseFormatter::json_success(&serde_json::json!({
                "summary_id": summary_id,
                "session_id": session_id.as_str(),
            })),
            Err(e) => Ok(to_contextual_tool_error(e)),
        }
    } else {
        match memory_service.get_session_summary(session_id).await {
            Ok(Some(summary)) => ResponseFormatter::json_success(&serde_json::json!({
                "session_id": summary.session_id,
                "topics": summary.topics,
                "decisions": summary.decisions,
                "next_steps": summary.next_steps,
                "key_files": summary.key_files,
                "created_at": summary.created_at,
            })),
            Ok(None) => Ok(CallToolResult::error(vec![Content::text(
                "Session summary not found",
            )])),
            Err(e) => Ok(to_contextual_tool_error(e)),
        }
    }
}
