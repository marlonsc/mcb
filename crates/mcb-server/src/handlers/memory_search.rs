//! Handler for the `memory_search` MCP tool (token-efficient index)
//!
//! This is Step 1 of the 3-layer progressive disclosure workflow:
//! 1. memory_search: Returns lightweight index (IDs, types, scores, previews)
//! 2. memory_timeline: Get chronological context around an anchor
//! 3. memory_get_observations: Fetch full details for specific IDs

use crate::args::MemorySearchArgs;
use mcb_application::ports::MemoryServiceInterface;
use mcb_domain::entities::memory::{MemoryFilter, ObservationType};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

pub struct MemorySearchHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

#[derive(Serialize)]
struct IndexResultItem {
    id: String,
    observation_type: String,
    relevance_score: f32,
    tags: Vec<String>,
    content_preview: String,
    session_id: Option<String>,
    repo_id: Option<String>,
    file_path: Option<String>,
    created_at: i64,
}

#[derive(Serialize)]
struct SearchIndexResults {
    query: String,
    count: usize,
    hint: String,
    results: Vec<IndexResultItem>,
}

impl MemorySearchHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemorySearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let observation_type = args
            .observation_type
            .as_ref()
            .map(|s| s.parse::<ObservationType>())
            .transpose()
            .map_err(|e: String| McpError::invalid_params(e, None))?;

        let filter = if args.tags.is_some()
            || observation_type.is_some()
            || args.session_id.is_some()
            || args.repo_id.is_some()
        {
            Some(MemoryFilter {
                tags: args.tags,
                observation_type,
                session_id: args.session_id,
                repo_id: args.repo_id,
                time_range: None,
                branch: None,
                commit: None,
            })
        } else {
            None
        };

        match self
            .memory_service
            .memory_search(&args.query, filter, args.limit)
            .await
        {
            Ok(results) => {
                let items: Vec<IndexResultItem> = results
                    .into_iter()
                    .map(|r| IndexResultItem {
                        id: r.id,
                        observation_type: r.observation_type,
                        relevance_score: r.relevance_score,
                        tags: r.tags,
                        content_preview: r.content_preview,
                        session_id: r.session_id,
                        repo_id: r.repo_id,
                        file_path: r.file_path,
                        created_at: r.created_at,
                    })
                    .collect();

                let response = SearchIndexResults {
                    query: args.query,
                    count: items.len(),
                    hint: "Use memory_timeline(anchor_id=ID) for context, memory_get_observations(ids=[...]) for full details".to_string(),
                    results: items,
                };

                let json = serde_json::to_string_pretty(&response)
                    .unwrap_or_else(|_| "Failed to serialize results".to_string());

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to search memories: {e}"
            ))])),
        }
    }
}
