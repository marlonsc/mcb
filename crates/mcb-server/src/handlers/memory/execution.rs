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
        let command = MemoryHelpers::get_required_str(data, "command")?;
        let exit_code = MemoryHelpers::get_i32(data, "exit_code").ok_or_else(|| {
            CallToolResult::error(vec![Content::text("Missing required field: exit_code")])
        })?;
        let duration_ms = MemoryHelpers::get_i64(data, "duration_ms").ok_or_else(|| {
            CallToolResult::error(vec![Content::text("Missing required field: duration_ms")])
        })?;
        let success = MemoryHelpers::get_bool(data, "success").ok_or_else(|| {
            CallToolResult::error(vec![Content::text("Missing required field: success")])
        })?;
        let execution_type_str = MemoryHelpers::get_required_str(data, "execution_type")?;
        let execution_type = MemoryHelpers::parse_execution_type(&execution_type_str)?;

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
                    let execution = result.observation.metadata.execution.as_ref()?;
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
