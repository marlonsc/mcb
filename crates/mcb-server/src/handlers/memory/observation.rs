use std::sync::Arc;

use mcb_domain::entities::memory::ObservationMetadata;
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::utils::vcs_context::VcsContext;
use mcb_domain::value_objects::ObservationId;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use uuid::Uuid;

use super::helpers::MemoryHelpers;
use crate::args::MemoryArgs;
use crate::formatter::ResponseFormatter;

pub async fn store_observation(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let data = match MemoryHelpers::json_map(&args.data) {
        Some(data) => data,
        None => {
            return Ok(CallToolResult::error(vec![Content::text(
                "Missing data payload for observation store",
            )]));
        }
    };
    let content = match MemoryHelpers::get_required_str(data, "content") {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let observation_type_str = match MemoryHelpers::get_required_str(data, "observation_type") {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let observation_type = match MemoryHelpers::parse_observation_type(&observation_type_str) {
        Ok(v) => v,
        Err(error_result) => return Ok(error_result),
    };
    let tags = MemoryHelpers::get_string_list(data, "tags");
    let vcs_context = VcsContext::capture();
    let metadata = ObservationMetadata {
        id: Uuid::new_v4().to_string(),
        session_id: None,
        repo_id: MemoryHelpers::get_str(data, "repo_id")
            .or_else(|| args.repo_id.clone())
            .or_else(|| vcs_context.repo_id.clone()),
        file_path: MemoryHelpers::get_str(data, "file_path"),
        branch: MemoryHelpers::get_str(data, "branch").or_else(|| vcs_context.branch.clone()),
        commit: MemoryHelpers::get_str(data, "commit").or_else(|| vcs_context.commit.clone()),
        execution: None,
        quality_gate: None,
    };
    match memory_service
        .store_observation(content, observation_type, tags, metadata)
        .await
    {
        Ok((observation_id, deduplicated)) => ResponseFormatter::json_success(&serde_json::json!({
            "observation_id": observation_id,
            "deduplicated": deduplicated,
        })),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Failed to store observation: {}",
            e
        ))])),
    }
}

pub async fn get_observations(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let ids = args.ids.clone().unwrap_or_default();
    if ids.is_empty() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Missing observation ids",
        )]));
    }
    match memory_service
        .get_observations_by_ids(
            &ids.iter()
                .map(|id| ObservationId::new(id.clone()))
                .collect::<Vec<_>>(),
        )
        .await
    {
        Ok(observations) => {
            let observations: Vec<_> = observations
                .into_iter()
                .map(|obs| {
                    serde_json::json!({
                        "id": obs.id,
                        "content": obs.content,
                        "observation_type": obs.observation_type.as_str(),
                        "tags": obs.tags,
                        "session_id": obs.metadata.session_id,
                        "repo_id": obs.metadata.repo_id,
                        "file_path": obs.metadata.file_path,
                        "branch": obs.metadata.branch,
                        "created_at": obs.created_at,
                        "content_hash": obs.content_hash,
                    })
                })
                .collect();
            ResponseFormatter::json_success(&serde_json::json!({
                "count": observations.len(),
                "observations": observations,
            }))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Failed to get observations: {}",
            e
        ))])),
    }
}
