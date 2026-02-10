use std::sync::Arc;

use mcb_domain::entities::memory::{
    MemoryFilter, ObservationMetadata, ObservationType, QualityGateResult,
};
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::utils::vcs_context::VcsContext;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use uuid::Uuid;

use super::helpers::MemoryHelpers;
use crate::args::MemoryArgs;
use crate::error_mapping::to_opaque_tool_error;
use crate::formatter::ResponseFormatter;

/// Stores a quality gate result as a semantic observation.
#[tracing::instrument(skip_all)]
pub async fn store_quality_gate(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let data = match MemoryHelpers::json_map(&args.data) {
        Some(data) => data,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing data payload for quality gate store",
            )]));
        }
    };
    let gate_name = match MemoryHelpers::get_required_str(data, "gate_name") {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let status_str = match MemoryHelpers::get_required_str(data, "status") {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let status = match MemoryHelpers::parse_quality_gate_status(&status_str) {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let timestamp =
        MemoryHelpers::get_i64(data, "timestamp").unwrap_or_else(|| chrono::Utc::now().timestamp());
    let quality_gate = QualityGateResult {
        id: Uuid::new_v4().to_string(),
        gate_name: gate_name.clone(),
        status,
        message: MemoryHelpers::get_str(data, "message"),
        timestamp,
        execution_id: MemoryHelpers::get_str(data, "execution_id"),
    };
    let vcs_context = VcsContext::capture();
    let content = format!(
        "Quality Gate: {} (status={})",
        gate_name,
        quality_gate.status.as_str()
    );
    let tags = vec![
        "quality_gate".to_string(),
        quality_gate.status.as_str().to_string(),
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
        execution: None,
        quality_gate: Some(quality_gate),
    };
    let project_id = args
        .project_id
        .clone()
        .or_else(|| MemoryHelpers::get_str(data, "project_id"))
        .unwrap_or_else(|| "default".to_string());

    match memory_service
        .store_observation(
            project_id,
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
        Err(e) => Ok(to_opaque_tool_error(e)),
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
        session_id: args.session_id.as_ref().map(|id| id.as_str().to_string()),
        repo_id: args.repo_id.clone(),
        time_range: None,
        branch: None,
        commit: None,
    };
    let query = "quality gate".to_string();
    let limit = args.limit.unwrap_or(10) as usize;
    let fetch_limit = limit * 5;
    match memory_service
        .search_memories(&query, Some(filter), fetch_limit)
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
        Err(e) => Ok(to_opaque_tool_error(e)),
    }
}
