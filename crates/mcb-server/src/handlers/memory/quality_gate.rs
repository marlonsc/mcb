use std::sync::Arc;

use mcb_domain::entities::memory::{MemoryFilter, ObservationType, QualityGateResult};
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::utils::compute_stable_id_hash;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::Value;

use super::common::{
    MemoryOriginOptions, build_observation_metadata, opt_str, require_data_map, require_str,
    resolve_memory_origin_context,
};
use crate::args::MemoryArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use uuid::Uuid;

/// Stores a quality gate result as a semantic observation.
#[tracing::instrument(skip_all)]
pub async fn store_quality_gate(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let data = match require_data_map(&args.data, "Missing data payload for quality gate store") {
        Ok(data) => data,
        Err(error_result) => return Ok(error_result),
    };
    let gate_name = match require_str(data, "gate_name") {
        Ok(value) => value,
        Err(error_result) => return Ok(error_result),
    };
    let status_str = match require_str(data, "status") {
        Ok(value) => value,
        Err(error_result) => return Ok(error_result),
    };
    let status: mcb_domain::entities::memory::QualityGateStatus = match status_str.parse() {
        Ok(v) => v,
        Err(error) => {
            return Ok(CallToolResult::error(vec![Content::text(
                error.to_string(),
            )]));
        }
    };
    let timestamp = data
        .get("timestamp")
        .and_then(Value::as_i64)
        .unwrap_or_else(|| chrono::Utc::now().timestamp());
    let quality_gate = QualityGateResult {
        id: Uuid::new_v4().to_string(),
        gate_name: gate_name.clone(),
        status,
        message: opt_str(data, "message"),
        timestamp,
        execution_id: opt_str(data, "execution_id"),
    };
    let content = format!(
        "Quality Gate: {} (status={})",
        gate_name,
        quality_gate.status.as_str()
    );
    let tags = vec![
        "quality_gate".to_string(),
        quality_gate.status.as_str().to_owned(),
    ];
    let origin = resolve_memory_origin_context(
        args,
        data,
        MemoryOriginOptions {
            execution_from_args: quality_gate.execution_id.as_deref(),
            execution_from_data: quality_gate.execution_id.as_deref(),
            file_path_payload: None,
            timestamp: Some(timestamp),
        },
    )?;

    let obs_metadata = build_observation_metadata(
        origin.canonical_session_id,
        origin.origin_context,
        None,
        Some(quality_gate),
    );

    match memory_service
        .store_observation(
            origin.project_id,
            content,
            ObservationType::QualityGate,
            tags,
            obs_metadata,
        )
        .await
    {
        Ok((observation_id, deduplicated)) => ResponseFormatter::json_success(&serde_json::json!({
            "observation_id": observation_id,
            "deduplicated": deduplicated,
        })),
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}

/// Retrieves stored quality gate results based on filters.
#[tracing::instrument(skip_all)]
pub async fn get_quality_gates(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let filter = MemoryFilter {
        id: None,
        project_id: args.project_id.clone(),
        tags: None,
        r#type: Some(ObservationType::QualityGate),
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
    let query = "quality gate";
    let limit = args.limit.unwrap_or(10) as usize;
    let fetch_limit = limit * 5;
    match memory_service
        .search_memories(query, Some(filter), fetch_limit)
        .await
    {
        Ok(results) => {
            let mut gates: Vec<_> = results
                .into_iter()
                .filter_map(|result| {
                    // Extract quality gate metadata from observation; skip if missing
                    // (None indicates observation is not a quality gate type)
                    if let Some(gate) = result.observation.metadata.quality_gate.as_ref() {
                        Some(serde_json::json!({
                            "observation_id": result.observation.id,
                            "gate_name": gate.gate_name,
                            "status": gate.status.as_str(),
                            "message": gate.message,
                            "timestamp": gate.timestamp,
                            "execution_id": gate.execution_id,
                            "created_at": result.observation.created_at,
                        }))
                    } else {
                        None
                    }
                })
                .collect();
            gates.sort_by(|a, b| {
                b.get("created_at")
                    .and_then(|v| v.as_i64())
                    .cmp(&a.get("created_at").and_then(|v| v.as_i64()))
            });
            gates.truncate(limit);
            ResponseFormatter::json_success(&serde_json::json!({
                "count": gates.len(),
                "quality_gates": gates,
            }))
        }
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}
