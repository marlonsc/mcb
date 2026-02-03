//! Handler for the `store_observation` MCP tool

use crate::args::StoreObservationArgs;
use mcb_application::ports::MemoryServiceInterface;
use mcb_domain::entities::memory::ObservationType;
use mcb_domain::utils::vcs_context::VcsContext;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub struct StoreObservationHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

#[derive(Serialize)]
struct StoreResult {
    observation_id: String,
    deduplicated: bool,
}

impl StoreObservationHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<StoreObservationArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let observation_type: ObservationType = args
            .observation_type
            .parse()
            .map_err(|e: String| McpError::invalid_params(e, None))?;

        let vcs_context = VcsContext::capture();

        let branch = args.branch.or(vcs_context.branch);
        let commit = args.commit.or(vcs_context.commit);

        match self
            .memory_service
            .store_observation(
                args.content.clone(),
                observation_type,
                args.tags,
                mcb_domain::entities::memory::ObservationMetadata {
                    id: Uuid::new_v4().to_string(),
                    session_id: args.session_id,
                    repo_id: args.repo_id.or(vcs_context.repo_id),
                    file_path: args.file_path,
                    branch,
                    commit,
                },
            )
            .await
        {
            Ok(id) => {
                let result = StoreResult {
                    observation_id: id,
                    deduplicated: false,
                };

                let json = serde_json::to_string_pretty(&result)
                    .unwrap_or_else(|_| "Failed to serialize result".to_string());

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to store observation: {e}"
            ))])),
        }
    }
}
