//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use std::sync::Arc;

use mcb_domain::ports::MemoryServiceInterface;
use mcb_domain::value_objects::ObservationId;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use super::common::build_memory_filter;
use crate::args::MemoryArgs;
use crate::error_mapping::{to_contextual_tool_error, to_opaque_mcp_error};
use crate::formatter::ResponseFormatter;
<<<<<<< HEAD
use crate::utils::mcp::{resolve_org_id, tool_error};
=======
use crate::utils::mcp::tool_error;
use mcb_utils::constants::keys::{FIELD_OBSERVATION_ID, FIELD_OBSERVATION_TYPE};
use mcb_utils::constants::limits::{DEFAULT_MEMORY_LIST_LIMIT, DEFAULT_TIMELINE_DEPTH};
>>>>>>> feat/v0.3.2-ci-gates

/// Lists semantic memories based on the provided search query and filters.
#[tracing::instrument(skip_all)]
pub async fn list_observations(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let filter = build_memory_filter(args, None, args.tags.clone());
    let limit = args.limit.unwrap_or(DEFAULT_MEMORY_LIST_LIMIT as u32) as usize;
    // INTENTIONAL: Optional query parameter; empty string means no filter
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
                        FIELD_OBSERVATION_TYPE: item.r#type.as_str(),
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

/// Resolve the timeline anchor id from `anchor_id` or, failing that, a `query` search.
///
/// The outer `Result` carries hard errors; the inner `Result` carries either the
/// resolved id or a ready-to-return tool error response (missing/empty anchor).
async fn resolve_timeline_anchor_id(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<Result<String, CallToolResult>, McpError> {
    if let Some(anchor_id) = args.anchor_id.clone() {
        return Ok(Ok(anchor_id));
    }
    let Some(query) = args.query.clone() else {
        return Ok(Err(tool_error("Missing anchor_id or query for timeline")));
    };
    let results = memory_service
        .search_memories(&query, None, 1)
        .await
        .map_err(|e| to_opaque_mcp_error(&e))?;
    match results.first() {
        Some(first) => Ok(Ok(first.observation.id.clone())),
        None => Ok(Err(tool_error("No anchor observation found"))),
    }
}

/// Retrieves a timeline of observations surrounding an anchor observation.
#[tracing::instrument(skip_all)]
pub async fn get_timeline(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let anchor_id = match resolve_timeline_anchor_id(memory_service, args).await? {
        Ok(anchor_id) => anchor_id,
        Err(response) => return Ok(response),
    };
    let filter = build_memory_filter(args, None, None);
    let depth_before = args.depth_before.unwrap_or(DEFAULT_TIMELINE_DEPTH);
    let depth_after = args.depth_after.unwrap_or(DEFAULT_TIMELINE_DEPTH);
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
                        FIELD_OBSERVATION_ID: observation.id,
                        "content": observation.content,
                        FIELD_OBSERVATION_TYPE: observation.r#type.as_str(),
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
