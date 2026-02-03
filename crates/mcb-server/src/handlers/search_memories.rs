//! Handler for the `search_memories` MCP tool

use crate::args::SearchMemoriesArgs;
use mcb_application::ports::MemoryServiceInterface;
use mcb_domain::entities::memory::{MemoryFilter, ObservationType};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

/// Handler for the MCP `search_memories` tool (semantic memory search).
pub struct SearchMemoriesHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

#[derive(Serialize)]
struct SearchResultItem {
    observation_id: String,
    content: String,
    observation_type: String,
    tags: Vec<String>,
    similarity_score: f32,
    session_id: Option<String>,
    created_at: i64,
}

#[derive(Serialize)]
struct SearchResults {
    query: String,
    count: usize,
    results: Vec<SearchResultItem>,
}

impl SearchMemoriesHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<SearchMemoriesArgs>,
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
            .search_memories(&args.query, filter, args.limit)
            .await
        {
            Ok(results) => {
                let items: Vec<SearchResultItem> = results
                    .into_iter()
                    .map(|r| SearchResultItem {
                        observation_id: r.observation.id,
                        content: r.observation.content,
                        observation_type: r.observation.observation_type.as_str().to_string(),
                        tags: r.observation.tags,
                        similarity_score: r.similarity_score,
                        session_id: r.observation.metadata.session_id,
                        created_at: r.observation.created_at,
                    })
                    .collect();

                let response = SearchResults {
                    query: args.query,
                    count: items.len(),
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
