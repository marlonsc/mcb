use std::sync::Arc;

use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::value_objects::ObservationId;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::common::build_memory_filter;
use crate::args::MemoryArgs;
use crate::error_mapping::{to_contextual_tool_error, to_opaque_mcp_error};
use crate::formatter::ResponseFormatter;
use crate::utils::mcp::tool_error;

/// Lists semantic memories based on the provided search query and filters.
#[tracing::instrument(skip_all)]
pub async fn list_observations(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let filter = build_memory_filter(args, None, args.tags.clone());
    let limit = args.limit.unwrap_or(10) as usize;
    let query = args.query.clone().unwrap_or_default();
    match memory_service
        .memory_search(&query, Some(filter), limit)
        .await
    {
        Ok(results) => {
            let items: Vec<_> = results
                .into_iter()
                .map(|item| {
                    serde_json::json!({
                        "id": item.id,
                        "observation_type": item.r#type.as_str(),
                        "relevance_score": item.relevance_score,
                        "tags": item.tags,
                        "content_preview": item.content_preview,
                        "session_id": item.session_id,
                        "repo_id": item.repo_id,
                        "file_path": item.file_path,
                        "created_at": item.created_at,
                    })
                })
                .collect();
            ResponseFormatter::json_success(&serde_json::json!({
                "query": query,
                "count": items.len(),
                "results": items,
                "hint": "Use memory action=timeline or memory action=get for details",
            }))
        }
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}

/// Retrieves a timeline of observations surrounding an anchor observation.
#[tracing::instrument(skip_all)]
pub async fn get_timeline(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let anchor_id = if let Some(anchor_id) = args.anchor_id.clone() {
        anchor_id
    } else if let Some(query) = args.query.clone() {
        let search_err = |e: mcb_domain::Error| to_opaque_mcp_error(e);
        let results = memory_service
            .search_memories(&query, None, 1)
            .await
            .map_err(search_err)?;
        if let Some(first) = results.first() {
            first.observation.id.clone()
        } else {
            return Ok(tool_error("No anchor observation found"));
        }
    } else {
        return Ok(tool_error("Missing anchor_id or query for timeline"));
    };
    let filter = build_memory_filter(args, None, None);
    let depth_before = args.depth_before.unwrap_or(5);
    let depth_after = args.depth_after.unwrap_or(5);
    match memory_service
        .get_timeline(
            &ObservationId::from_string(&anchor_id),
            depth_before,
            depth_after,
            Some(filter),
        )
        .await
    {
        Ok(timeline) => {
            let items: Vec<_> = timeline
                .into_iter()
                .map(|observation| {
                    serde_json::json!({
                        "observation_id": observation.id,
                        "content": observation.content,
                        "observation_type": observation.r#type.as_str(),
                        "created_at": observation.created_at,
                    })
                })
                .collect();
            ResponseFormatter::json_success(&serde_json::json!({
                "anchor_id": anchor_id,
                "count": items.len(),
                "timeline": items,
            }))
        }
        Err(e) => Ok(to_contextual_tool_error(e)),
    }
}
