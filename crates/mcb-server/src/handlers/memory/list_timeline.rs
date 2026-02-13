use std::sync::Arc;

use mcb_domain::entities::memory::MemoryFilter;
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::value_objects::ObservationId;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::args::MemoryArgs;
use crate::error_mapping::{to_contextual_tool_error, to_opaque_mcp_error};
use crate::formatter::ResponseFormatter;

/// Lists semantic memories based on the provided search query and filters.
#[tracing::instrument(skip_all)]
pub async fn list_observations(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let filter = MemoryFilter {
        id: None,
        project_id: args.project_id.clone(),
        tags: args.tags.clone(),
        r#type: None,
        session_id: args.session_id.as_ref().map(|id| id.as_str().to_string()),
        parent_session_id: None,
        repo_id: args.repo_id.clone(),
        time_range: None,
        branch: None,
        commit: None,
    };
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
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Failed to list memories: {}",
            e
        ))])),
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
            return Ok(CallToolResult::error(vec![Content::text(
                "No anchor observation found",
            )]));
        }
    } else {
        return Ok(CallToolResult::error(vec![Content::text(
            "Missing anchor_id or query for timeline",
        )]));
    };
    let filter = MemoryFilter {
        id: None,
        project_id: args.project_id.clone(),
        tags: None,
        r#type: None,
        session_id: args.session_id.as_ref().map(|id| id.as_str().to_string()),
        parent_session_id: None,
        repo_id: args.repo_id.clone(),
        time_range: None,
        branch: None,
        commit: None,
    };
    let depth_before = args.depth_before.unwrap_or(5);
    let depth_after = args.depth_after.unwrap_or(5);
    match memory_service
        .get_timeline(
            &ObservationId::new(&anchor_id),
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
