//! Search handler for code and memory search operations.

use crate::args::{SearchArgs, SearchResource};
use crate::collection_mapping::map_collection_name;
use crate::formatter::ResponseFormatter;
use mcb_application::domain_services::search::SearchServiceInterface;
use mcb_application::ports::MemoryServiceInterface;
use mcb_domain::entities::memory::MemoryFilter;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use std::sync::Arc;
use std::time::Instant;
use validator::Validate;

/// Handler for code and memory search MCP tool operations.
#[derive(Clone)]
pub struct SearchHandler {
    search_service: Arc<dyn SearchServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
}

impl SearchHandler {
    pub fn new(
        search_service: Arc<dyn SearchServiceInterface>,
        memory_service: Arc<dyn MemoryServiceInterface>,
    ) -> Self {
        Self {
            search_service,
            memory_service,
        }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = args.validate() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Invalid arguments: {}",
                e
            ))]));
        }

        let query = args.query.trim();
        if query.is_empty() {
            return Ok(CallToolResult::error(vec![Content::text(
                "Query cannot be empty",
            )]));
        }

        match args.resource {
            SearchResource::Code => {
                let collection_name = args.collection.as_deref().unwrap_or("default");
                let milvus_collection = match map_collection_name(collection_name) {
                    Ok(name) => name,
                    Err(e) => {
                        return Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to map collection name '{}': {}",
                            collection_name, e
                        ))]));
                    }
                };
                let timer = Instant::now();
                let limit = args.limit.unwrap_or(10) as usize;
                match self
                    .search_service
                    .search(&milvus_collection, query, limit)
                    .await
                {
                    Ok(results) => ResponseFormatter::format_search_response(
                        query,
                        &results,
                        timer.elapsed(),
                        limit,
                    ),
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                        "Search error: {}",
                        e
                    ))])),
                }
            }
            SearchResource::Memory => {
                let filter = MemoryFilter {
                    tags: args.tags.clone(),
                    observation_type: None,
                    session_id: args.session_id.clone(),
                    repo_id: None,
                    time_range: None,
                    branch: None,
                    commit: None,
                    id: None,
                };
                let limit = args.limit.unwrap_or(10) as usize;
                match self
                    .memory_service
                    .search_memories(query, Some(filter), limit)
                    .await
                {
                    Ok(results) => {
                        let results: Vec<_> = results
                            .into_iter()
                            .map(|r| {
                                serde_json::json!({
                                    "observation_id": r.observation.id,
                                    "content": r.observation.content,
                                    "observation_type": r.observation.observation_type.as_str(),
                                    "tags": r.observation.tags,
                                    "similarity_score": r.similarity_score,
                                    "session_id": r.observation.metadata.session_id,
                                })
                            })
                            .collect();
                        let response = ResponseFormatter::json_success(&serde_json::json!({
                            "query": query,
                            "count": results.len(),
                            "results": results,
                        }))
                        .map_err(|e| {
                            McpError::internal_error(
                                format!("Failed to format memory search results: {e}"),
                                None,
                            )
                        })?;
                        Ok(response)
                    }
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                        "Memory search error: {}",
                        e
                    ))])),
                }
            }
        }
    }
}
