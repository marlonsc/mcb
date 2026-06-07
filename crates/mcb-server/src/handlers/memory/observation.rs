//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use std::sync::Arc;

use mcb_domain::ports::MemoryServiceInterface;
use mcb_domain::value_objects::ObservationId;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::common::{
    MemoryOriginOptions, build_observation_metadata, require_data_map, require_str,
    resolve_memory_origin_context, str_vec,
};
use crate::args::MemoryArgs;
use crate::constants::fields::{
    FIELD_BRANCH, FIELD_COUNT, FIELD_OBSERVATION_ID, FIELD_OBSERVATION_TYPE, FIELD_OBSERVATIONS,
};
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::utils::mcp::tool_error;

/// Stores a new semantic observation with the provided content, type, and tags.
#[tracing::instrument(skip_all)]
pub async fn store_observation(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let data = extract_field!(require_data_map(
        &args.data,
        "Missing data payload for observation store"
    ));
    let content = extract_field!(require_str(data, "content"));
    let observation_type_str = extract_field!(require_str(data, "observation_type"));
    let observation_type: mcb_domain::entities::memory::ObservationType =
        parse_enum!(observation_type_str, "observation_type");
    let tags = str_vec(data, "tags");
    let origin = resolve_memory_origin_context(
        args,
        data,
        &MemoryOriginOptions {
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
            FIELD_OBSERVATION_ID: observation_id,
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
    let ids = match args.ids.clone() {
        Some(ids) => ids,
        None => return Ok(tool_error("Missing required field: ids")),
    };
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
                        FIELD_OBSERVATION_TYPE: obs.r#type.as_str(),
                        "tags": obs.tags,
                        "session_id": obs.metadata.session_id,
                        "repo_id": obs.metadata.repo_id,
                        "file_path": obs.metadata.file_path,
                        (FIELD_BRANCH): obs.metadata.branch,
                        "created_at": obs.created_at,
                        "content_hash": obs.content_hash,
                    })
                })
                .collect();
            ResponseFormatter::json_success(&serde_json::json!({
                (FIELD_COUNT): observations.len(),
                (FIELD_OBSERVATIONS): observations,
            }))
        }
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}
