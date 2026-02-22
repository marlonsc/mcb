//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use std::sync::Arc;

use mcb_domain::entities::memory::{MemorySearchResult, ObservationType, QualityGateResult};
use mcb_domain::ports::MemoryServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::Value;

use super::common::{
    MemoryOriginOptions, SearchMemoriesJsonSpec, build_observation_metadata, opt_str,
    require_data_map, require_str, resolve_memory_origin_context, search_memories_as_json,
};
use crate::args::MemoryArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use mcb_domain::utils::id as domain_id;

use crate::constants::fields::{FIELD_MESSAGE, FIELD_OBSERVATION_ID, TAG_QUALITY_GATE};

/// Stores a quality gate result as a semantic observation.
#[tracing::instrument(skip_all)]
pub async fn store_quality_gate(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let data = extract_field!(require_data_map(
        &args.data,
        "Missing data payload for quality gate store"
    ));
    let gate_name = extract_field!(require_str(data, "gate_name"));
    let status_str = extract_field!(require_str(data, "status"));
    let status: mcb_domain::entities::memory::QualityGateStatus = parse_enum!(status_str, "status");
    let timestamp = data
        .get("timestamp")
        .and_then(Value::as_i64)
        .unwrap_or_else(|| chrono::Utc::now().timestamp());
    let quality_gate = QualityGateResult {
        id: domain_id::generate().to_string(),
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
        TAG_QUALITY_GATE.to_owned(),
        quality_gate.status.as_str().to_owned(),
    ];
    let origin = resolve_memory_origin_context(
        args,
        data,
        &MemoryOriginOptions {
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
            FIELD_OBSERVATION_ID: observation_id,
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
    search_memories_as_json(
        memory_service,
        args,
        SearchMemoriesJsonSpec {
            query: "quality gate",
            obs_type: ObservationType::QualityGate,
            result_key: "quality_gates",
            mapper: |result: &MemorySearchResult| {
                let gate = result.observation.metadata.quality_gate.as_ref()?;
                Some(serde_json::json!({
                    FIELD_OBSERVATION_ID: result.observation.id,
                    "gate_name": gate.gate_name,
                    "status": gate.status.as_str(),
                    FIELD_MESSAGE: gate.message,
                    "timestamp": gate.timestamp,
                    "execution_id": gate.execution_id,
                    "created_at": result.observation.created_at,
                }))
            },
        },
    )
    .await
}
