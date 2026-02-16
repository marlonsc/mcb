//! Search handler for code and memory search operations.

use std::sync::Arc;
use std::time::Instant;

use mcb_domain::entities::memory::MemoryFilter;
use mcb_domain::error::Error;
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::ports::services::SearchServiceInterface;
use mcb_domain::utils::id as domain_id;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use validator::Validate;

use crate::error_mapping::safe_internal_error;

use crate::args::{SearchArgs, SearchResource};
use crate::constants::fields::{
    FIELD_BRANCH, FIELD_COMMIT, FIELD_COUNT, FIELD_OBSERVATION_ID, FIELD_OBSERVATION_TYPE,
    FIELD_QUERY, FIELD_RESULTS,
};
use crate::constants::limits::DEFAULT_SEARCH_LIMIT;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::utils::collections::normalize_collection_name;

/// Handler for code and memory search MCP tool operations.
#[derive(Clone)]
pub struct SearchHandler {
    search_service: Arc<dyn SearchServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
}

impl SearchHandler {
    /// Create a new `SearchHandler`.
    pub fn new(
        search_service: Arc<dyn SearchServiceInterface>,
        memory_service: Arc<dyn MemoryServiceInterface>,
    ) -> Self {
        Self {
            search_service,
            memory_service,
        }
    }

    /// Handle a search tool request.
    ///
    /// # Errors
    /// Returns an error when required request parameters are invalid.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = args.validate() {
            return Ok(to_contextual_tool_error(Error::invalid_argument(
                e.to_string(),
            )));
        }

        let query = args.query.trim();
        if query.is_empty() {
            return Ok(to_contextual_tool_error(Error::invalid_argument(
                "Query cannot be empty",
            )));
        }

        match args.resource {
            SearchResource::Code => {
                let collection_name = args.collection.as_deref().ok_or_else(|| {
                    McpError::invalid_params("collection parameter is required", None)
                })?;
                let collection_id = match normalize_collection_name(collection_name) {
                    Ok(id) => id,
                    Err(reason) => {
                        return Ok(to_contextual_tool_error(Error::invalid_argument(reason)));
                    }
                };
                let timer = Instant::now();
                let limit = args.limit.unwrap_or(DEFAULT_SEARCH_LIMIT as u32) as usize;
                match self
                    .search_service
                    .search(&collection_id, query, limit)
                    .await
                {
                    Ok(results) => ResponseFormatter::format_search_response(
                        query,
                        &results,
                        timer.elapsed(),
                        limit,
                    ),
                    Err(e) => Ok(to_contextual_tool_error(e)),
                }
            }
            SearchResource::Memory | SearchResource::Context => {
                let filter = MemoryFilter {
                    tags: args.tags.clone(),
                    r#type: if matches!(args.resource, SearchResource::Context) {
                        Some(mcb_domain::entities::memory::ObservationType::Context)
                    } else {
                        None
                    },
                    session_id: args.session_id.map(|id| {
                        let id_str = id.to_string();
                        domain_id::correlate_id("session", &id_str)
                    }),
                    ..Default::default()
                };
                let limit = args.limit.unwrap_or(DEFAULT_SEARCH_LIMIT as u32) as usize;
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
                                    FIELD_OBSERVATION_ID: r.observation.id,
                                    "project_id": r.observation.project_id,
                                    "content": r.observation.content,
                                    FIELD_OBSERVATION_TYPE: r.observation.r#type.as_str(),
                                    "tags": r.observation.tags,
                                    "similarity_score": r.similarity_score,
                                    "session_id": r.observation.metadata.session_id.clone(),
                                    "repo_id": r.observation.metadata.repo_id.clone(),
                                    "file_path": r.observation.metadata.file_path,
                                    (FIELD_BRANCH): r.observation.metadata.branch,
                                    (FIELD_COMMIT): r.observation.metadata.commit,
                                    "origin_context": r.observation.metadata.origin_context,
                                })
                            })
                            .collect();
                        let response = ResponseFormatter::json_success(&serde_json::json!({
                            (FIELD_QUERY): query,
                            (FIELD_COUNT): results.len(),
                            (FIELD_RESULTS): results,
                        }))
                        .map_err(|e| safe_internal_error("format memory search results", &e))?;
                        Ok(response)
                    }
                    Err(e) => Ok(to_contextual_tool_error(e)),
                }
            }
        }
    }
}
