use std::sync::Arc;

use mcb_domain::entities::memory::{
    ExecutionMetadata, MemoryFilter, ObservationMetadata, ObservationType,
};
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::utils::vcs_context::VcsContext;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use tracing::error;
use uuid::Uuid;

use super::helpers::MemoryHelpers;
use crate::args::MemoryArgs;
use crate::formatter::ResponseFormatter;

/// Store an execution observation in memory
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
    let command = match MemoryHelpers::get_required_str(data, "command") {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let exit_code = match MemoryHelpers::get_i32(data, "exit_code").ok_or_else(|| {
        CallToolResult::error(vec![Content::text("Missing required field: exit_code")])
    }) {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let duration_ms = match MemoryHelpers::get_i64(data, "duration_ms").ok_or_else(|| {
        CallToolResult::error(vec![Content::text("Missing required field: duration_ms")])
    }) {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let success = match MemoryHelpers::get_bool(data, "success").ok_or_else(|| {
        CallToolResult::error(vec![Content::text("Missing required field: success")])
    }) {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let execution_type_str = match MemoryHelpers::get_required_str(data, "execution_type") {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let execution_type = match MemoryHelpers::parse_execution_type(&execution_type_str) {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let metadata = ExecutionMetadata {
        id: Uuid::new_v4().to_string(),
        command: command.clone(),
        exit_code: Some(exit_code),
        duration_ms: Some(duration_ms),
        success,
        execution_type,
        coverage: MemoryHelpers::get_f32(data, "coverage"),
        files_affected: MemoryHelpers::get_string_list(data, "files_affected"),
        output_summary: MemoryHelpers::get_str(data, "output_summary"),
        warnings_count: MemoryHelpers::get_i32(data, "warnings_count"),
        errors_count: MemoryHelpers::get_i32(data, "errors_count"),
    };
    let vcs_context = VcsContext::capture();
    let content = format!(
        "Execution: {} (exit_code={}, success={})",
        command, exit_code, success
    );
    let tags = vec![
        "execution".to_string(),
        metadata.execution_type.as_str().to_string(),
        if success { "success" } else { "failure" }.to_string(),
    ];
    let obs_metadata = ObservationMetadata {
        id: Uuid::new_v4().to_string(),
        session_id: MemoryHelpers::get_str(data, "session_id")
            .or_else(|| args.session_id.as_ref().map(|id| id.as_str().to_string())),
        repo_id: MemoryHelpers::get_str(data, "repo_id")
            .or_else(|| args.repo_id.clone())
            .or_else(|| vcs_context.repo_id.clone()),
        file_path: None,
        branch: MemoryHelpers::get_str(data, "branch").or_else(|| vcs_context.branch.clone()),
        commit: MemoryHelpers::get_str(data, "commit").or_else(|| vcs_context.commit.clone()),
        execution: Some(metadata),
        quality_gate: None,
    };
    match memory_service
        .store_observation(content, ObservationType::Execution, tags, obs_metadata)
        .await
    {
        Ok((observation_id, deduplicated)) => ResponseFormatter::json_success(&serde_json::json!({
            "observation_id": observation_id,
            "deduplicated": deduplicated,
        })),
        Err(e) => {
            error!(error = %e, "Failed to store execution");
            Ok(CallToolResult::error(vec![Content::text(
                "Failed to store execution",
            )]))
        }
    }
}

/// Retrieve execution observations filtered by session and repo
pub async fn get_executions(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let filter = MemoryFilter {
        id: None,
        tags: None,
        observation_type: Some(ObservationType::Execution),
        session_id: args.session_id.as_ref().map(|id| id.as_str().to_string()),
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
                    let execution = result.observation.metadata.execution?;
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
        Err(e) => {
            error!(error = %e, "Failed to get executions");
            Ok(CallToolResult::error(vec![Content::text(
                "Failed to get executions",
            )]))
        }
    }
}
