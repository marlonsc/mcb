//! Handler for the `memory_get_observations` MCP tool

use crate::args::MemoryGetObservationsArgs;
use mcb_application::ports::MemoryServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

pub struct MemoryGetObservationsHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

#[derive(Serialize)]
struct ObservationDetail {
    observation_id: String,
    content: String,
    observation_type: String,
    tags: Vec<String>,
    session_id: Option<String>,
    repo_id: Option<String>,
    file_path: Option<String>,
    branch: Option<String>,
    created_at: i64,
    content_hash: String,
}

#[derive(Serialize)]
struct GetObservationsResponse {
    count: usize,
    observations: Vec<ObservationDetail>,
}

impl MemoryGetObservationsHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryGetObservationsArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        match self.memory_service.get_observations_by_ids(&args.ids).await {
            Ok(observations) => {
                let details: Vec<ObservationDetail> = observations
                    .into_iter()
                    .map(|obs| ObservationDetail {
                        observation_id: obs.id,
                        content: obs.content,
                        observation_type: obs.observation_type.as_str().to_string(),
                        tags: obs.tags,
                        session_id: obs.metadata.session_id,
                        repo_id: obs.metadata.repo_id,
                        file_path: obs.metadata.file_path,
                        branch: obs.metadata.branch,
                        created_at: obs.created_at,
                        content_hash: obs.content_hash,
                    })
                    .collect();

                let response = GetObservationsResponse {
                    count: details.len(),
                    observations: details,
                };

                let json = serde_json::to_string_pretty(&response)
                    .unwrap_or_else(|_| "Failed to serialize results".to_string());

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to get observations: {e}"
            ))])),
        }
    }
}
