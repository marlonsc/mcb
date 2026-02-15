use std::sync::Arc;

use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::value_objects::ObservationId;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::common::{
    MemoryOriginOptions, build_observation_metadata, require_data_map, require_str,
    resolve_memory_origin_context, str_vec,
};
use crate::args::MemoryArgs;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::utils::mcp::tool_error;

/// Stores a new semantic observation with the provided content, type, and tags.
#[tracing::instrument(skip_all)]
pub async fn store_observation(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let data = match require_data_map(&args.data, "Missing data payload for observation store") {
        Ok(data) => data,
        Err(error_result) => return Ok(error_result),
    };
    let content = match require_str(data, "content") {
        Ok(value) => value,
        Err(error_result) => return Ok(error_result),
    };
    // Consider using mcb_domain::schema::memory::COL_OBSERVATION_TYPE instead.
    let observation_type_str: String = match require_str(data, "observation_type") {
        Ok(value) => value,
        Err(error_result) => return Ok(error_result),
    };
    let observation_type: mcb_domain::entities::memory::ObservationType =
        match observation_type_str.parse() {
            Ok(v) => v,
            Err(_) => {
                return Ok(tool_error(format!(
                    "Unknown observation type: {observation_type_str}"
                )));
            }
        };
    let tags = str_vec(data, "tags");
    let origin = resolve_memory_origin_context(
        args,
        data,
        MemoryOriginOptions {
            execution_from_args: None,
            execution_from_data: None,
            file_path_payload: data.get("file_path").and_then(|v| v.as_str()),
            timestamp: None,
        },
    )?;

    let metadata = build_observation_metadata(
        origin.canonical_session_id,
        origin.origin_context,
        None,
        None,
    );
    match memory_service
        .store_observation(origin.project_id, content, observation_type, tags, metadata)
        .await
    {
        Ok((observation_id, deduplicated)) => ResponseFormatter::json_success(&serde_json::json!({
            "observation_id": observation_id,
            "deduplicated": deduplicated,
        })),
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}

/// Retrieves semantic observations by their unique identifiers.
#[tracing::instrument(skip_all)]
pub async fn get_observations(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let ids = args.ids.clone().unwrap_or_default();
    if ids.is_empty() {
        return Ok(tool_error("Missing observation ids"));
    }
    match memory_service
        .get_observations_by_ids(
            &ids.iter()
                .map(|id| ObservationId::from_string(id))
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
                        "observation_type": obs.r#type.as_str(),
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
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}
