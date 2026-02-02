//! Handler for the `memory_inject_context` MCP tool

use crate::args::MemoryInjectContextArgs;
use mcb_application::ports::MemoryServiceInterface;
use mcb_domain::entities::memory::{MemoryFilter, ObservationType};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use validator::Validate;

pub struct MemoryInjectContextHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

#[derive(Serialize)]
struct ContextObservation {
    id: String,
    content: String,
    observation_type: String,
    created_at: i64,
}

#[derive(Serialize)]
struct ContextBundle {
    session_id: String,
    observation_count: usize,
    observation_ids: Vec<String>,
    context: String,
    estimated_tokens: usize,
}

impl MemoryInjectContextHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryInjectContextArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

        let observation_types: Result<Vec<ObservationType>, String> = args
            .observation_types
            .iter()
            .map(|s| s.parse::<ObservationType>())
            .collect();

        let _observation_types =
            observation_types.map_err(|e| McpError::invalid_params(e, None))?;

        let filter = MemoryFilter {
            session_id: Some(args.session_id.clone()),
            repo_id: args.repo_id,
            observation_type: None,
            tags: None,
            time_range: None,
            branch: None,
            commit: None,
        };

        match self
            .memory_service
            .search_memories("", Some(filter), args.limit)
            .await
        {
            Ok(results) => {
                let observations: Vec<ContextObservation> = results
                    .iter()
                    .map(|r| ContextObservation {
                        id: r.observation.id.clone(),
                        content: r.observation.content.clone(),
                        observation_type: r.observation.observation_type.as_str().to_string(),
                        created_at: r.observation.created_at,
                    })
                    .collect();

                let observation_ids: Vec<String> =
                    observations.iter().map(|o| o.id.clone()).collect();

                let mut context_parts: Vec<String> = Vec::new();
                let mut total_chars = 0;
                let max_chars = args.max_tokens.map(|t| t * 4).unwrap_or(usize::MAX);

                for obs in &observations {
                    let part = format!(
                        "[{}] {}: {}",
                        obs.observation_type.to_uppercase(),
                        obs.id,
                        obs.content
                    );
                    if total_chars + part.len() > max_chars {
                        break;
                    }
                    total_chars += part.len();
                    context_parts.push(part);
                }

                let context = context_parts.join("\n\n");
                let estimated_tokens = context.len() / 4;

                let response = ContextBundle {
                    session_id: args.session_id,
                    observation_count: observation_ids.len(),
                    observation_ids,
                    context,
                    estimated_tokens,
                };

                let json = serde_json::to_string_pretty(&response)
                    .unwrap_or_else(|_| "Failed to serialize results".to_string());

                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to inject context: {e}"
            ))])),
        }
    }
}
