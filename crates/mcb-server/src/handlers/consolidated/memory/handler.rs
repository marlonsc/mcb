//! Memory handler implementation.

use crate::args::{MemoryAction, MemoryArgs, MemoryResource};
use mcb_domain::ports::services::MemoryServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use std::sync::Arc;
use validator::Validate;

use super::{execution, inject, list_timeline, observation, quality_gate, session};

/// Handler for memory-related MCP tool operations.
///
/// Supports storing and retrieving observations, executions, quality gates,
/// and session summaries with semantic search capabilities.
#[derive(Clone)]
pub struct MemoryHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

impl MemoryHandler {
    /// Creates a new MemoryHandler with the given memory service.
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    /// Handles a memory tool invocation.
    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;

        match args.action {
            MemoryAction::Store => self.handle_store(&args).await,
            MemoryAction::Get => self.handle_get(&args).await,
            MemoryAction::List => self.handle_list(&args).await,
            MemoryAction::Timeline => self.handle_timeline(&args).await,
            MemoryAction::Inject => self.handle_inject(&args).await,
        }
    }

    async fn handle_store(&self, args: &MemoryArgs) -> Result<CallToolResult, McpError> {
        match args.resource {
            MemoryResource::Observation => {
                observation::store_observation(&self.memory_service, args).await
            }
            MemoryResource::Execution => {
                execution::store_execution(&self.memory_service, args).await
            }
            MemoryResource::QualityGate => {
                quality_gate::store_quality_gate(&self.memory_service, args).await
            }
            MemoryResource::ErrorPattern => Ok(CallToolResult::error(vec![Content::text(
                "Error pattern memory is not implemented yet",
            )])),
            MemoryResource::Session => session::store_session(&self.memory_service, args).await,
        }
    }

    async fn handle_get(&self, args: &MemoryArgs) -> Result<CallToolResult, McpError> {
        match args.resource {
            MemoryResource::Observation => {
                observation::get_observations(&self.memory_service, args).await
            }
            MemoryResource::Execution => {
                execution::get_executions(&self.memory_service, args).await
            }
            MemoryResource::QualityGate => {
                quality_gate::get_quality_gates(&self.memory_service, args).await
            }
            MemoryResource::ErrorPattern => Ok(CallToolResult::error(vec![Content::text(
                "Error pattern memory is not implemented yet",
            )])),
            MemoryResource::Session => session::get_session(&self.memory_service, args).await,
        }
    }

    async fn handle_list(&self, args: &MemoryArgs) -> Result<CallToolResult, McpError> {
        match args.resource {
            MemoryResource::Observation => {
                list_timeline::list_observations(&self.memory_service, args).await
            }
            _ => Ok(CallToolResult::error(vec![Content::text(
                "List action is only supported for observation resource",
            )])),
        }
    }

    async fn handle_timeline(&self, args: &MemoryArgs) -> Result<CallToolResult, McpError> {
        list_timeline::get_timeline(&self.memory_service, args).await
    }

    async fn handle_inject(&self, args: &MemoryArgs) -> Result<CallToolResult, McpError> {
        inject::inject_context(&self.memory_service, args).await
    }
}
