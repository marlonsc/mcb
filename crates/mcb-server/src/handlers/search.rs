//! Search handler for code and memory search operations.

use std::sync::Arc;
use std::time::Instant;

use mcb_domain::entities::memory::MemoryFilter;
use mcb_domain::error::Error;
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::ports::services::SearchServiceInterface;
use mcb_domain::utils::compute_stable_id_hash;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use validator::Validate;

use crate::args::{SearchArgs, SearchResource};
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::handlers::helpers::resolve_org_id;
use crate::utils::collections::normalize_collection_name;

/// Handler for code and memory search MCP tool operations.
#[derive(Clone)]
pub struct SearchHandler {
    search_service: Arc<dyn SearchServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
}

impl SearchHandler {
    /// Create a new SearchHandler.
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

        let _org_id = resolve_org_id(args.org_id.as_deref());

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
                let limit = args.limit.unwrap_or(10) as usize;
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
                    session_id: args
                        .session_id
                        .clone()
                        .map(|id| compute_stable_id_hash("session", id.as_str())),
                    parent_session_id: None,
                    repo_id: None,
                    time_range: None,
                    branch: None,
                    commit: None,
                    id: None,
                    project_id: None,
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
                                    "project_id": r.observation.project_id,
                                    "content": r.observation.content,
                                    // TODO(ORG002): Duplicate string literal "observation_type".
                                    "observation_type": r.observation.r#type.as_str(),
                                    "tags": r.observation.tags,
                                    "similarity_score": r.similarity_score,
                                    "session_id": r.observation.metadata.session_id.clone(),
                                    "repo_id": r.observation.metadata.repo_id.clone(),
                                    "file_path": r.observation.metadata.file_path,
                                    "branch": r.observation.metadata.branch,
                                    "commit": r.observation.metadata.commit,
                                    "origin_context": r.observation.metadata.origin_context,
                                })
                            })
                            .collect();
                        let fmt_err = |_e: McpError| {
                            McpError::internal_error("failed to format memory search results", None)
                        };
                        let response = ResponseFormatter::json_success(&serde_json::json!({
                            "query": query,
                            "count": results.len(),
                            "results": results,
                        }))
                        .map_err(fmt_err)?;
                        Ok(response)
                    }
                    Err(e) => Ok(to_contextual_tool_error(e)),
                }
            }
        }
    }
}
