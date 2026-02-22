//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Memory handler implementation.

use std::sync::Arc;

use mcb_domain::entities::memory::ErrorPattern;
use mcb_domain::ports::MemoryServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use validator::Validate;

use super::{execution, inject, list_timeline, observation, quality_gate, session};
use crate::args::{MemoryAction, MemoryArgs, MemoryResource};
use crate::constants::fields::FIELD_COUNT;
use crate::constants::limits::DEFAULT_MEMORY_LIMIT;
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::utils::json;
use crate::utils::mcp::{resolve_identifier_precedence, tool_error};

/// Handler for memory-related MCP tool operations.
///
/// Supports storing and retrieving observations, executions, quality gates,
/// and session summaries with semantic search capabilities.
#[derive(Clone)]
pub struct MemoryHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

handler_new!(MemoryHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
});

impl MemoryHandler {
    /// Handles a memory tool invocation.
    ///
    /// # Errors
    /// Returns an error when argument validation fails.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate().map_err(|e| {
            McpError::invalid_params(format!("failed to validate memory args: {e}"), None)
        })?;

        match args.action {
            MemoryAction::Store => self.handle_store(&args).await,
            MemoryAction::Get => self.handle_get(&args).await,
            MemoryAction::List => self.handle_list(&args).await,
            MemoryAction::Timeline => self.handle_timeline(&args).await,
            MemoryAction::Inject => self.handle_inject(&args).await,
        }
    }

    async fn handle_store(&self, args: &MemoryArgs) -> Result<CallToolResult, McpError> {
        self.dispatch_resource(args, MemoryResourceAction::Store)
            .await
    }

    async fn handle_get(&self, args: &MemoryArgs) -> Result<CallToolResult, McpError> {
        self.dispatch_resource(args, MemoryResourceAction::Get)
            .await
    }

    async fn dispatch_resource(
        &self,
        args: &MemoryArgs,
        action: MemoryResourceAction,
    ) -> Result<CallToolResult, McpError> {
        match (action, args.resource) {
            (MemoryResourceAction::Store, MemoryResource::Observation) => {
                observation::store_observation(&self.memory_service, args).await
            }
            (MemoryResourceAction::Store, MemoryResource::Execution) => {
                execution::store_execution(&self.memory_service, args).await
            }
            (MemoryResourceAction::Store, MemoryResource::QualityGate) => {
                quality_gate::store_quality_gate(&self.memory_service, args).await
            }
            (MemoryResourceAction::Store, MemoryResource::Session) => {
                session::store_session(&self.memory_service, args).await
            }
            (MemoryResourceAction::Store, MemoryResource::ErrorPattern) => {
                self.handle_store_error_pattern(args).await
            }
            (MemoryResourceAction::Get, MemoryResource::Observation) => {
                observation::get_observations(&self.memory_service, args).await
            }
            (MemoryResourceAction::Get, MemoryResource::Execution) => {
                execution::get_executions(&self.memory_service, args).await
            }
            (MemoryResourceAction::Get, MemoryResource::QualityGate) => {
                quality_gate::get_quality_gates(&self.memory_service, args).await
            }
            (MemoryResourceAction::Get, MemoryResource::Session) => {
                session::get_session(&self.memory_service, args).await
            }
            (MemoryResourceAction::Get, MemoryResource::ErrorPattern) => {
                self.handle_get_error_pattern(args).await
            }
        }
    }

    async fn handle_store_error_pattern(
        &self,
        args: &MemoryArgs,
    ) -> Result<CallToolResult, McpError> {
        let data = match json::json_map(&args.data) {
            Some(data) => data,
            None => {
                return Ok(tool_error("Missing data payload for error pattern store"));
            }
        };

        // Validate and deserialize ErrorPattern from data
        // We assume the data matches ErrorPattern structure or we build it manually
        // For simplicity, let's try to deserialize the whole data object
        let pattern: ErrorPattern =
            match serde_json::from_value(serde_json::Value::Object(data.clone())) {
                Ok(p) => p,
                Err(e) => {
                    return Ok(to_contextual_tool_error(e));
                }
            };
        let resolved_project_id = resolve_identifier_precedence(
            "project_id",
            args.project_id.as_deref(),
            Some(pattern.project_id.as_str()),
        )?
        .ok_or_else(|| {
            McpError::invalid_params("project_id is required for error pattern store", None)
        })?;
        let pattern = ErrorPattern {
            project_id: resolved_project_id,
            ..pattern
        };

        match self.memory_service.store_error_pattern(pattern).await {
            Ok(id) => ResponseFormatter::json_success(&serde_json::json!({
                "id": id,
            })),
            Err(e) => Ok(to_contextual_tool_error(e)),
        }
    }

    async fn handle_get_error_pattern(
        &self,
        args: &MemoryArgs,
    ) -> Result<CallToolResult, McpError> {
        let project_id = args.project_id.clone().ok_or_else(|| {
            McpError::invalid_params("project_id is required for error pattern search", None)
        })?;

        let query = args.query.clone().unwrap_or_default();
        let limit = args.limit.unwrap_or(DEFAULT_MEMORY_LIMIT as u32) as usize;

        match self
            .memory_service
            .search_error_patterns(&query, project_id, limit)
            .await
        {
            Ok(patterns) => ResponseFormatter::json_success(&serde_json::json!({
                (FIELD_COUNT): patterns.len(),
                "patterns": patterns,
            })),
            Err(e) => Ok(to_contextual_tool_error(e)),
        }
    }

    async fn handle_list(&self, args: &MemoryArgs) -> Result<CallToolResult, McpError> {
        match args.resource {
            MemoryResource::Observation => {
                list_timeline::list_observations(&self.memory_service, args).await
            }
            MemoryResource::Execution
            | MemoryResource::QualityGate
            | MemoryResource::ErrorPattern
            | MemoryResource::Session => Ok(tool_error(
                "List action is only supported for observation resource",
            )),
        }
    }

    async fn handle_timeline(&self, args: &MemoryArgs) -> Result<CallToolResult, McpError> {
        list_timeline::get_timeline(&self.memory_service, args).await
    }

    async fn handle_inject(&self, args: &MemoryArgs) -> Result<CallToolResult, McpError> {
        inject::inject_context(&self.memory_service, args).await
    }
}

#[derive(Clone, Copy)]
enum MemoryResourceAction {
    Store,
    Get,
}
