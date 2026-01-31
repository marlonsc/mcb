//! MCP Server Implementation
//!
//! Core MCP protocol server that orchestrates semantic code search operations.
//! Follows Clean Architecture principles with dependency injection.

use std::sync::Arc;

use rmcp::ErrorData as McpError;
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolResult, Implementation, ListToolsResult, PaginatedRequestParams, ProtocolVersion,
    ServerCapabilities, ServerInfo,
};

use mcb_application::{
    ContextServiceInterface, IndexingServiceInterface, SearchServiceInterface,
    ValidationServiceInterface,
};

use crate::handlers::{
    ClearIndexHandler, GetIndexingStatusHandler, IndexCodebaseHandler, SearchCodeHandler,
    ValidateArchitectureHandler,
};
use crate::tools::{ToolHandlers, create_tool_list, route_tool_call};

/// Core MCP server implementation
///
/// This server implements the MCP protocol for semantic code search.
/// It depends only on domain services and receives all dependencies through
/// constructor injection following Clean Architecture principles.
#[derive(Clone)]
pub struct McpServer {
    /// Domain services for core operations
    services: McpServices,
    /// Tool handlers for MCP protocol
    handlers: ToolHandlers,
}

/// Domain services container (keeps struct field count manageable)
#[derive(Clone)]
struct McpServices {
    indexing: Arc<dyn IndexingServiceInterface>,
    context: Arc<dyn ContextServiceInterface>,
    search: Arc<dyn SearchServiceInterface>,
}

impl McpServer {
    /// Create a new MCP server with injected dependencies
    pub fn new(
        indexing_service: Arc<dyn IndexingServiceInterface>,
        context_service: Arc<dyn ContextServiceInterface>,
        search_service: Arc<dyn SearchServiceInterface>,
        validation_service: Arc<dyn ValidationServiceInterface>,
    ) -> Self {
        let handlers = ToolHandlers {
            index_codebase: Arc::new(IndexCodebaseHandler::new(indexing_service.clone())),
            search_code: Arc::new(SearchCodeHandler::new(search_service.clone())),
            get_indexing_status: Arc::new(GetIndexingStatusHandler::new(indexing_service.clone())),
            clear_index: Arc::new(ClearIndexHandler::new(indexing_service.clone())),
            validate_architecture: Arc::new(ValidateArchitectureHandler::new(validation_service)),
        };

        Self {
            services: McpServices {
                indexing: indexing_service,
                context: context_service,
                search: search_service,
            },
            handlers,
        }
    }

    /// Access to indexing service
    pub fn indexing_service(&self) -> Arc<dyn IndexingServiceInterface> {
        Arc::clone(&self.services.indexing)
    }

    /// Access to context service
    pub fn context_service(&self) -> Arc<dyn ContextServiceInterface> {
        Arc::clone(&self.services.context)
    }

    /// Access to search service
    pub fn search_service(&self) -> Arc<dyn SearchServiceInterface> {
        Arc::clone(&self.services.search)
    }

    /// Access to index codebase handler (for HTTP transport)
    pub fn index_codebase_handler(&self) -> Arc<IndexCodebaseHandler> {
        Arc::clone(&self.handlers.index_codebase)
    }

    /// Access to search code handler (for HTTP transport)
    pub fn search_code_handler(&self) -> Arc<SearchCodeHandler> {
        Arc::clone(&self.handlers.search_code)
    }

    /// Access to get indexing status handler (for HTTP transport)
    pub fn get_indexing_status_handler(&self) -> Arc<GetIndexingStatusHandler> {
        Arc::clone(&self.handlers.get_indexing_status)
    }

    /// Access to clear index handler (for HTTP transport)
    pub fn clear_index_handler(&self) -> Arc<ClearIndexHandler> {
        Arc::clone(&self.handlers.clear_index)
    }

    /// Access to validate architecture handler (for HTTP transport)
    pub fn validate_architecture_handler(&self) -> Arc<ValidateArchitectureHandler> {
        Arc::clone(&self.handlers.validate_architecture)
    }
}

impl ServerHandler for McpServer {
    /// Get server information and capabilities
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26, // Updated to latest MCP protocol
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "MCP Context Browser".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "MCP Context Browser - Semantic Code Search\n\n\
                 AI-powered code understanding for semantic search across large codebases.\n\n\
                 Tools:\n\
                 - index_codebase: Build a semantic index for a directory\n\
                 - search_code: Query indexed code using natural language\n\
                 - get_indexing_status: Inspect indexing progress\n\
                 - clear_index: Clear a collection before re-indexing\n\
                 - validate_architecture: Run architecture validation rules on a codebase\n"
                    .to_string(),
            ),
        }
    }

    /// List available tools
    async fn list_tools(
        &self,
        _pagination: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = create_tool_list()?;
        Ok(ListToolsResult {
            tools,
            meta: Default::default(),
            next_cursor: None,
        })
    }

    /// Call a tool
    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        route_tool_call(request, &self.handlers).await
    }
}
