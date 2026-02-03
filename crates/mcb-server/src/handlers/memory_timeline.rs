//! Handler for the `memory_timeline` MCP tool

use crate::args::MemoryTimelineArgs;
use mcb_application::ports::MemoryServiceInterface;
use mcb_domain::entities::memory::{MemoryFilter, ObservationType};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

/// Handler for the MCP `memory_timeline` tool (progressive disclosure step 2).
pub struct MemoryTimelineHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

#[derive(Serialize)]
struct TimelineItem {
    observation_id: String,
    content: String,
    observation_type: String,
    created_at: i64,
    is_anchor: bool,
}

#[derive(Serialize)]
struct TimelineResponse {
    anchor_id: String,
    count: usize,
    timeline: Vec<TimelineItem>,
}

impl MemoryTimelineHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryTimelineArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let anchor_id = match (args.anchor_id, args.query) {
            (Some(id), _) => id,
            (None, Some(query)) => {
                let results = self
                    .memory_service
                    .search_memories(&query, None, 1)
                    .await
                    .map_err(|_| McpError::internal_error("Search failed", None))?;

                results
                    .first()
                    .map(|r| r.observation.id.clone())
                    .ok_or_else(|| {
                        McpError::invalid_params("No observations found for query", None)
                    })?
            }
            (None, None) => {
                return Err(McpError::invalid_params(
                    "Either anchor_id or query must be provided",
                    None,
                ));
            }
        };

        let observation_type = args
            .observation_type
            .as_ref()
            .map(|s| s.parse::<ObservationType>())
            .transpose()
            .map_err(|e: String| McpError::invalid_params(e.as_str(), None))?;

        let filter =
            if args.session_id.is_some() || args.repo_id.is_some() || observation_type.is_some() {
                Some(MemoryFilter {
                    session_id: args.session_id,
                    repo_id: args.repo_id,
                    observation_type,
                    tags: None,
                    time_range: None,
                    branch: None,
                    commit: None,
                })
            } else {
                None
            };

        match self
            .memory_service
            .get_timeline(&anchor_id, args.depth_before, args.depth_after, filter)
            .await
        {
            Ok(observations) => {
                let items: Vec<TimelineItem> = observations
                    .iter()
                    .map(|obs| TimelineItem {
                        observation_id: obs.id.clone(),
                        content: obs.content.clone(),
                        observation_type: obs.observation_type.as_str().to_string(),
                        created_at: obs.created_at,
                        is_anchor: obs.id == anchor_id,
                    })
                    .collect();

                let response = TimelineResponse {
                    anchor_id,
                    count: items.len(),
                    timeline: items,
                };

                let json = serde_json::to_string_pretty(&response)
                    .unwrap_or_else(|_| "Failed to serialize results".to_string());

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to get timeline: {e}"
            ))])),
        }
    }
}
