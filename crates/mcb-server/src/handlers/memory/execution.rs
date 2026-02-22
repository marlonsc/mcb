//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use std::sync::Arc;

use mcb_domain::entities::memory::{ExecutionMetadata, MemorySearchResult, ObservationType};
use mcb_domain::ports::MemoryServiceInterface;
use mcb_domain::utils::id as domain_id;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::Value;

use super::common::{
    MemoryOriginOptions, SearchMemoriesJsonSpec, build_observation_metadata, opt_str, require_bool,
    require_data_map, require_i32, require_i64, require_str, resolve_memory_origin_context,
    search_memories_as_json, str_vec,
};
use crate::args::MemoryArgs;
use crate::constants::fields::{FIELD_OBSERVATION_ID, TAG_EXECUTION, TAG_FAILURE, TAG_SUCCESS};
use crate::formatter::ResponseFormatter;
use crate::utils::mcp::tool_error;

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
        let command = require_str(data, "command")?;
        let exit_code = require_i32(data, "exit_code")?;
        let duration_ms = require_i64(data, "duration_ms")?;
        let success = require_bool(data, "success")?;
        let execution_type_str = require_str(data, "execution_type")?;

        let execution_type = execution_type_str
            .parse()
            .map_err(|_| tool_error(format!("Invalid execution_type: {execution_type_str}")))?;

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
#[tracing::instrument(skip_all)]
pub async fn store_execution(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let data = extract_field!(require_data_map(
        &args.data,
        "Missing data payload for execution store"
    ));
    let validated = extract_field!(ValidatedExecutionData::validate(data));
    let metadata = ExecutionMetadata {
        id: domain_id::generate().to_string(),
        command: validated.command.clone(),
        exit_code: Some(validated.exit_code),
        duration_ms: Some(validated.duration_ms),
        success: validated.success,
        execution_type: validated.execution_type,
        coverage: data
            .get("coverage")
            .and_then(Value::as_f64)
            .map(|value| value as f32),
        files_affected: str_vec(data, "files_affected"),
        output_summary: data
            .get("output_summary")
            .and_then(Value::as_str)
            .map(str::to_owned),
        warnings_count: data
            .get("warnings_count")
            .and_then(Value::as_i64)
            .and_then(|value| value.try_into().ok()),
        errors_count: data
            .get("errors_count")
            .and_then(Value::as_i64)
            .and_then(|value| value.try_into().ok()),
    };
    let content = format!(
        "Execution: {} (exit_code={}, success={})",
        validated.command, validated.exit_code, validated.success
    );
    let tags = vec![
        TAG_EXECUTION.to_owned(),
        metadata.execution_type.as_str().to_owned(),
        if validated.success {
            TAG_SUCCESS
        } else {
            TAG_FAILURE
        }
        .to_owned(),
    ];
    let payload_execution_id = opt_str(data, "execution_id");
    let generated_execution_id = metadata.id.clone();

    let origin = resolve_memory_origin_context(
        args,
        data,
        &MemoryOriginOptions {
            execution_from_args: Some(generated_execution_id.as_str()),
            execution_from_data: payload_execution_id.as_deref(),
            file_path_payload: None,
            timestamp: None,
        },
    )
    .map_err(|err| {
        McpError::invalid_params(
            format!("failed to resolve execution origin context: {err}"),
            None,
        )
    })?;

    let obs_metadata = build_observation_metadata(
        origin.canonical_session_id,
        origin.origin_context,
        Some(metadata),
        None,
    );

    match memory_service
        .store_observation(
            origin.project_id,
            content,
            ObservationType::Execution,
            tags,
            obs_metadata,
        )
        .await
    {
        Ok((observation_id, deduplicated)) => ResponseFormatter::json_success(&serde_json::json!({
            FIELD_OBSERVATION_ID: observation_id,
            "deduplicated": deduplicated,
        })),
        Err(e) => {
            mcb_domain::error!("store_execution", "Failed to store execution", &e);
            Ok(tool_error("Failed to store execution"))
        }
    }
}

/// Retrieve execution observations filtered by session and repo
#[tracing::instrument(skip_all)]
pub async fn get_executions(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    search_memories_as_json(
        memory_service,
        args,
        SearchMemoriesJsonSpec {
            query: "execution",
            obs_type: ObservationType::Execution,
            result_key: "executions",
            mapper: |result: &MemorySearchResult| {
                let execution = result.observation.metadata.execution.as_ref()?;
                Some(serde_json::json!({
                    FIELD_OBSERVATION_ID: result.observation.id,
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
            },
        },
    )
    .await
}
