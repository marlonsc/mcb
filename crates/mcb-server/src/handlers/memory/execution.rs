use std::sync::Arc;

use mcb_domain::entities::memory::{
    ExecutionMetadata, MemoryFilter, ObservationMetadata, ObservationType,
};
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::utils::{compute_stable_id_hash, vcs_context::VcsContext};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use tracing::error;
use uuid::Uuid;

use super::helpers::MemoryHelpers;
use crate::args::MemoryArgs;
use crate::formatter::ResponseFormatter;
use crate::handler_helpers::{OriginContextInput, resolve_origin_context};

/// Validated execution data extracted from JSON payload
struct ValidatedExecutionData {
    command: String,
    exit_code: i32,
    duration_ms: i64,
    success: bool,
    execution_type: mcb_domain::entities::memory::ExecutionType,
}

impl ValidatedExecutionData {
    /// Validate and extract all required fields from JSON data
    fn validate(data: &serde_json::Map<String, serde_json::Value>) -> Result<Self, CallToolResult> {
        let command = MemoryHelpers::get_required_str(data, "command").map_err(|_| {
            CallToolResult::error(vec![Content::text("Missing required field: command")])
        })?;

        let exit_code = MemoryHelpers::get_i32(data, "exit_code").ok_or_else(|| {
            CallToolResult::error(vec![Content::text("Missing required field: exit_code")])
        })?;

        let duration_ms = MemoryHelpers::get_i64(data, "duration_ms").ok_or_else(|| {
            CallToolResult::error(vec![Content::text("Missing required field: duration_ms")])
        })?;

        let success = MemoryHelpers::get_bool(data, "success").ok_or_else(|| {
            CallToolResult::error(vec![Content::text("Missing required field: success")])
        })?;

        let execution_type_str =
            MemoryHelpers::get_required_str(data, "execution_type").map_err(|_| {
                CallToolResult::error(vec![Content::text(
                    "Missing required field: execution_type",
                )])
            })?;

        let execution_type =
            MemoryHelpers::parse_execution_type(&execution_type_str).map_err(|_| {
                CallToolResult::error(vec![Content::text(format!(
                    "Invalid execution_type: {}",
                    execution_type_str
                ))])
            })?;

        Ok(Self {
            command,
            exit_code,
            duration_ms,
            success,
            execution_type,
        })
    }
}

/// Store an execution observation in memory
// TODO(KISS005): Function store_execution is too long (154 lines, max: 50).
// Consider splitting into smaller helpers for data extraction, origin context resolution, and storage.
#[tracing::instrument(skip_all)]
pub async fn store_execution(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let data = match MemoryHelpers::json_map(&args.data) {
        Some(data) => data,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing data payload for execution store",
            )]));
        }
    };

    // Validate all required fields upfront
    let validated = match ValidatedExecutionData::validate(data) {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let metadata = ExecutionMetadata {
        id: Uuid::new_v4().to_string(),
        command: validated.command.clone(),
        exit_code: Some(validated.exit_code),
        duration_ms: Some(validated.duration_ms),
        success: validated.success,
        execution_type: validated.execution_type,
        coverage: MemoryHelpers::get_f32(data, "coverage"),
        files_affected: MemoryHelpers::get_string_list(data, "files_affected"),
        output_summary: MemoryHelpers::get_str(data, "output_summary"),
        warnings_count: MemoryHelpers::get_i32(data, "warnings_count"),
        errors_count: MemoryHelpers::get_i32(data, "errors_count"),
    };
    let vcs_context = VcsContext::capture();
    let content = format!(
        "Execution: {} (exit_code={}, success={})",
        validated.command, validated.exit_code, validated.success
    );
    let tags = vec![
        "execution".to_string(),
        metadata.execution_type.as_str().to_string(),
        if validated.success {
            "success"
        } else {
            "failure"
        }
        .to_string(),
    ];
    let arg_session_id = args
        .session_id
        .clone()
        .map(|id| compute_stable_id_hash("session", id.as_str()));
    let canonical_session_id = arg_session_id.clone();
    let parent_session_hash = args
        .parent_session_id
        .clone()
        .map(|id| compute_stable_id_hash("parent_session", id.as_str()));
    let payload_repo_id = MemoryHelpers::get_str(data, "repo_id");
    let payload_project_id = MemoryHelpers::get_str(data, "project_id");
    let payload_branch = MemoryHelpers::get_str(data, "branch");
    let payload_commit = MemoryHelpers::get_str(data, "commit");
    let payload_repo_path = MemoryHelpers::get_str(data, "repo_path");
    let payload_worktree_id = MemoryHelpers::get_str(data, "worktree_id");
    let payload_operator_id = MemoryHelpers::get_str(data, "operator_id");
    let payload_machine_id = MemoryHelpers::get_str(data, "machine_id");
    let payload_agent_program = MemoryHelpers::get_str(data, "agent_program");
    let payload_model_id = MemoryHelpers::get_str(data, "model_id");
    let payload_delegated = MemoryHelpers::get_bool(data, "delegated");
    let payload_execution_id = MemoryHelpers::get_str(data, "execution_id");
    let generated_execution_id = metadata.id.clone();

    let mut origin_context = resolve_origin_context(OriginContextInput {
        org_id: args.org_id.as_deref(),
        project_id_args: args.project_id.as_deref(),
        project_id_payload: payload_project_id.as_deref(),
        session_from_args: None,
        session_from_data: None,
        parent_session_from_args: None,
        parent_session_from_data: None,
        execution_from_args: Some(generated_execution_id.as_str()),
        execution_from_data: payload_execution_id.as_deref(),
        tool_name_args: Some("memory"),
        tool_name_payload: None,
        repo_id_args: args.repo_id.as_deref(),
        repo_id_payload: payload_repo_id.as_deref(),
        repo_path_args: None,
        repo_path_payload: payload_repo_path.as_deref(),
        worktree_id_args: None,
        worktree_id_payload: payload_worktree_id.as_deref(),
        file_path_args: None,
        file_path_payload: None,
        branch_args: None,
        branch_payload: payload_branch.as_deref(),
        commit_args: None,
        commit_payload: payload_commit.as_deref(),
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
    })
    // TODO(ERR001): Missing error context. Add .context() or .map_err() for better error messages.
    ?;
    if origin_context.repo_id.is_none() {
        origin_context.repo_id = vcs_context.repo_id.clone();
    }
    if origin_context.branch.is_none() {
        origin_context.branch = vcs_context.branch.clone();
    }
    if origin_context.commit.is_none() {
        origin_context.commit = vcs_context.commit.clone();
    }
    origin_context.session_id = None;
    origin_context.session_id_hash = canonical_session_id.clone();
    origin_context.parent_session_id = None;
    origin_context.parent_session_id_hash = parent_session_hash;
    let project_id = origin_context.project_id.clone().ok_or_else(|| {
        // TODO(ERR001): Missing error context. Consider adding more descriptive details or mapping to a specific error variant.
        McpError::invalid_params("project_id is required for execution store", None)
    })?;

    let obs_metadata = ObservationMetadata {
        id: Uuid::new_v4().to_string(),
        session_id: canonical_session_id,
        repo_id: origin_context.repo_id.clone(),
        file_path: None,
        branch: origin_context.branch.clone(),
        commit: origin_context.commit.clone(),
        execution: Some(metadata),
        quality_gate: None,
        origin_context: Some(origin_context),
    };

    match memory_service
        .store_observation(
            project_id,
            content,
            ObservationType::Execution,
            tags,
            obs_metadata,
        )
        .await
    {
        Ok((observation_id, deduplicated)) => ResponseFormatter::json_success(&serde_json::json!({
            "observation_id": observation_id,
            "deduplicated": deduplicated,
        })),
        Err(_e) => {
            error!("Failed to store execution");
            Ok(CallToolResult::error(vec![Content::text(
                "Failed to store execution",
            )]))
        }
    }
}

/// Retrieve execution observations filtered by session and repo
// TODO(KISS005): Function get_executions is too long (68 lines, max: 50).
// Consider extracting the result mapping and sorting into dedicated functions.
#[tracing::instrument(skip_all)]
pub async fn get_executions(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let filter = MemoryFilter {
        id: None,
        project_id: args.project_id.clone(),
        tags: None,
        r#type: Some(ObservationType::Execution),
        session_id: args
            .session_id
            .clone()
            .map(|id| compute_stable_id_hash("session", id.as_str())),
        parent_session_id: args.parent_session_id.clone(),
        repo_id: args.repo_id.clone(),
        time_range: None,
        branch: None,
        commit: None,
    };
    let query = "execution".to_string();
    let limit = args.limit.unwrap_or(10) as usize;
    let fetch_limit = limit * 5;
    match memory_service
        .search_memories(&query, Some(filter), fetch_limit)
        .await
    {
        Ok(results) => {
            let mut executions: Vec<_> = results
                .into_iter()
                .filter_map(|result| {
                    // Extract execution metadata from observation; skip if missing
                    // (None indicates observation is not an execution type)
                    if let Some(execution) = result.observation.metadata.execution.as_ref() {
                        Some(serde_json::json!({
                                "observation_id": result.observation.id,
                                "command": execution.command,
                            "exit_code": execution.exit_code,
                            "duration_ms": execution.duration_ms,
                            "success": execution.success,
                            "execution_type": execution.execution_type.as_str(),
                            "coverage": execution.coverage,
                            "files_affected": execution.files_affected,
                            "output_summary": execution.output_summary,
                            "warnings_count": execution.warnings_count,
                            "errors_count": execution.errors_count,
                            "created_at": result.observation.created_at,
                        }))
                    } else {
                        None
                    }
                })
                .collect();
            executions.sort_by(|a, b| {
                b.get("created_at")
                    .and_then(|v| v.as_i64())
                    .cmp(&a.get("created_at").and_then(|v| v.as_i64()))
            });
            executions.truncate(limit);
            ResponseFormatter::json_success(&serde_json::json!({
                "count": executions.len(),
                "executions": executions,
            }))
        }
        Err(_e) => {
            error!("Failed to get executions");
            Ok(CallToolResult::error(vec![Content::text(
                "Failed to get executions",
            )]))
        }
    }
}
