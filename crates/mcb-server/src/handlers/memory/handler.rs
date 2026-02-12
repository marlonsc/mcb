//! Memory handler implementation.

use std::sync::Arc;

use mcb_application::services::RepositoryResolver;
use mcb_domain::entities::memory::ErrorPattern;
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_domain::value_objects::OrgContext;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use validator::Validate;

use super::helpers::MemoryHelpers;
use super::{execution, inject, list_timeline, observation, quality_gate, session};
use crate::args::{MemoryAction, MemoryArgs, MemoryResource};
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;

/// Handler for memory-related MCP tool operations.
///
/// Supports storing and retrieving observations, executions, quality gates,
/// and session summaries with semantic search capabilities.
#[derive(Clone)]
pub struct MemoryHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
    resolver: Arc<RepositoryResolver>,
}

impl MemoryHandler {
    pub fn new(
        memory_service: Arc<dyn MemoryServiceInterface>,
        resolver: Arc<RepositoryResolver>,
    ) -> Self {
        Self {
            memory_service,
            resolver,
        }
    }

    /// Handles a memory tool invocation.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryArgs>,
    ) -> Result<CallToolResult, McpError> {
        let validate_err = |_e: validator::ValidationErrors| {
            McpError::invalid_params("failed to validate memory args", None)
        };
        args.validate().map_err(validate_err)?;

        let org_ctx = OrgContext::current();
        let org_id = org_ctx.id_str();
        let project_id = self.resolver.resolve_project_id(org_id).await;

        match args.action {
            MemoryAction::Store => self.handle_store(&args, &project_id).await,
            MemoryAction::Get => self.handle_get(&args, &project_id).await,
            MemoryAction::List => self.handle_list(&args, &project_id).await,
            MemoryAction::Timeline => self.handle_timeline(&args, &project_id).await,
            MemoryAction::Inject => self.handle_inject(&args, &project_id).await,
        }
    }

    async fn handle_store(
        &self,
        args: &MemoryArgs,
        project_id: &str,
    ) -> Result<CallToolResult, McpError> {
        match args.resource {
            MemoryResource::Observation => {
                observation::store_observation(&self.memory_service, args, project_id).await
            }
            MemoryResource::Execution => {
                execution::store_execution(&self.memory_service, args, project_id).await
            }
            MemoryResource::QualityGate => {
                quality_gate::store_quality_gate(&self.memory_service, args, project_id).await
            }
            MemoryResource::ErrorPattern => self.handle_store_error_pattern(args).await,
            MemoryResource::Session => session::store_session(&self.memory_service, args).await,
        }
    }

    async fn handle_get(
        &self,
        args: &MemoryArgs,
        project_id: &str,
    ) -> Result<CallToolResult, McpError> {
        match args.resource {
            MemoryResource::Observation => {
                observation::get_observations(&self.memory_service, args).await
            }
            MemoryResource::Execution => {
                execution::get_executions(&self.memory_service, args, project_id).await
            }
            MemoryResource::QualityGate => {
                quality_gate::get_quality_gates(&self.memory_service, args, project_id).await
            }
            MemoryResource::ErrorPattern => self.handle_get_error_pattern(args, project_id).await,
            MemoryResource::Session => session::get_session(&self.memory_service, args).await,
        }
    }

    async fn handle_list(
        &self,
        args: &MemoryArgs,
        project_id: &str,
    ) -> Result<CallToolResult, McpError> {
        match args.resource {
            MemoryResource::Observation => {
                list_timeline::list_observations(&self.memory_service, args, project_id).await
            }
            _ => Ok(CallToolResult::error(vec![Content::text(
                "List action is only supported for observation resource",
            )])),
        }
    }

    async fn handle_timeline(
        &self,
        args: &MemoryArgs,
        project_id: &str,
    ) -> Result<CallToolResult, McpError> {
        list_timeline::get_timeline(&self.memory_service, args, project_id).await
    }

    async fn handle_inject(
        &self,
        args: &MemoryArgs,
        project_id: &str,
    ) -> Result<CallToolResult, McpError> {
        inject::inject_context(&self.memory_service, args, project_id).await
    }

    async fn handle_store_error_pattern(
        &self,
        args: &MemoryArgs,
    ) -> Result<CallToolResult, McpError> {
        let data = match MemoryHelpers::json_map(&args.data) {
            Some(data) => data,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(
                    "Missing data payload for error pattern store",
                )]));
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
        project_id: &str,
    ) -> Result<CallToolResult, McpError> {
        let project_id = project_id.to_string();

        let query = args.query.clone().unwrap_or_default();
        let limit = args.limit.unwrap_or(10) as usize;

        match self
            .memory_service
            .search_error_patterns(&query, project_id, limit)
            .await
        {
            Ok(patterns) => ResponseFormatter::json_success(&serde_json::json!({
                "count": patterns.len(),
                "patterns": patterns,
            })),
            Err(e) => Ok(to_contextual_tool_error(e)),
        }
    }
}
