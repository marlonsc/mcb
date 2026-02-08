//!
//! Core MCP protocol server that orchestrates semantic code search operations.
//! Follows Clean Architecture principles with dependency injection.

use std::sync::Arc;

use mcb_domain::ports::providers::VcsProvider;
use mcb_domain::ports::services::AgentSessionServiceInterface;
use mcb_domain::ports::services::{
    ContextServiceInterface, IndexingServiceInterface, MemoryServiceInterface,
    ProjectDetectorService, SearchServiceInterface, ValidationServiceInterface,
};
use rmcp::model::{
    CallToolResult, Implementation, ListToolsResult, PaginatedRequestParams, ProtocolVersion,
    ServerCapabilities, ServerInfo,
};
use rmcp::ErrorData as McpError;
use rmcp::ServerHandler;

use crate::handlers::{
    AgentHandler, IndexHandler, MemoryHandler, ProjectHandler, SearchHandler, SessionHandler,
    ValidateHandler, VcsHandler,
};
use crate::hooks::HookProcessor;
use crate::tools::{create_tool_list, route_tool_call, ToolHandlers};

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
pub struct McpServices {
    /// Indexing service
    pub indexing: Arc<dyn IndexingServiceInterface>,
    /// Context service
    pub context: Arc<dyn ContextServiceInterface>,
    /// Search service
    pub search: Arc<dyn SearchServiceInterface>,
    /// Validation service
    pub validation: Arc<dyn ValidationServiceInterface>,
    /// Memory service
    pub memory: Arc<dyn MemoryServiceInterface>,
    /// Agent session service
    pub agent_session: Arc<dyn AgentSessionServiceInterface>,
    /// Project detector service
    pub project: Arc<dyn ProjectDetectorService>,
    /// VCS provider
    pub vcs: Arc<dyn VcsProvider>,
}

impl McpServer {
    /// Create a new MCP server with injected dependencies
    pub fn new(services: McpServices) -> Self {
        let hook_processor = HookProcessor::new(Some(services.memory.clone()));

        let handlers = ToolHandlers {
            index: Arc::new(IndexHandler::new(services.indexing.clone())),
            search: Arc::new(SearchHandler::new(
                services.search.clone(),
                services.memory.clone(),
            )),
            validate: Arc::new(ValidateHandler::new(services.validation.clone())),
            memory: Arc::new(MemoryHandler::new(services.memory.clone())),
            session: Arc::new(SessionHandler::new(
                services.agent_session.clone(),
                services.memory.clone(),
            )),
            agent: Arc::new(AgentHandler::new(services.agent_session.clone())),
            project: Arc::new(ProjectHandler::new()),
            vcs: Arc::new(VcsHandler::new(services.vcs.clone())),
            hook_processor: Arc::new(hook_processor),
        };

        Self { services, handlers }
    }

    /// Create a new MCP server from domain services
    /// This is the preferred constructor that uses the DI container
    pub fn from_services(services: McpServices) -> Self {
        Self::new(services)
    }

    /// Access to indexing service
    pub fn indexing_service(&self) -> Arc<dyn IndexingServiceInterface> {
        Arc::clone(&self.services.indexing)
    }

    /// Access to context service
    pub fn context_service(&self) -> Arc<dyn ContextServiceInterface> {
        Arc::clone(&self.services.context)
    }

    /// Access to VCS provider (for branch/repo handlers)
    pub fn vcs_provider(&self) -> Arc<dyn VcsProvider> {
        Arc::clone(&self.services.vcs)
    }

    /// Access to search service
    pub fn search_service(&self) -> Arc<dyn SearchServiceInterface> {
        Arc::clone(&self.services.search)
    }

    /// Access to validation service
    pub fn validation_service(&self) -> Arc<dyn ValidationServiceInterface> {
        Arc::clone(&self.services.validation)
    }

    /// Access to memory service
    pub fn memory_service(&self) -> Arc<dyn MemoryServiceInterface> {
        Arc::clone(&self.services.memory)
    }

    /// Access to agent session service
    pub fn agent_session_service(&self) -> Arc<dyn AgentSessionServiceInterface> {
        Arc::clone(&self.services.agent_session)
    }

    /// Access to project service
    pub fn project_service(&self) -> Arc<dyn ProjectDetectorService> {
        Arc::clone(&self.services.project)
    }

    /// Access to index handler (for HTTP transport)
    pub fn index_handler(&self) -> Arc<IndexHandler> {
        Arc::clone(&self.handlers.index)
    }

    /// Access to search handler (for HTTP transport)
    pub fn search_handler(&self) -> Arc<SearchHandler> {
        Arc::clone(&self.handlers.search)
    }

    /// Access to validate handler (for HTTP transport)
    pub fn validate_handler(&self) -> Arc<ValidateHandler> {
        Arc::clone(&self.handlers.validate)
    }

    /// Access to memory handler (for HTTP transport)
    pub fn memory_handler(&self) -> Arc<MemoryHandler> {
        Arc::clone(&self.handlers.memory)
    }

    /// Access to session handler (for HTTP transport)
    pub fn session_handler(&self) -> Arc<SessionHandler> {
        Arc::clone(&self.handlers.session)
    }

    /// Access to agent handler (for HTTP transport)
    pub fn agent_handler(&self) -> Arc<AgentHandler> {
        Arc::clone(&self.handlers.agent)
    }

    /// Access to VCS handler (for HTTP transport)
    pub fn vcs_handler(&self) -> Arc<VcsHandler> {
        Arc::clone(&self.handlers.vcs)
    }

    /// Access to project handler (for HTTP transport)
    pub fn project_handler(&self) -> Arc<ProjectHandler> {
        Arc::clone(&self.handlers.project)
    }

    /// Access to hook processor (for automatic memory operations)
    pub fn hook_processor(&self) -> Arc<HookProcessor> {
        Arc::clone(&self.handlers.hook_processor)
    }
}

impl ServerHandler for McpServer {
    /// Get server information and capabilities
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "MCP Context Browser".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                r#"MCP Context Browser - Semantic Code Search

tools:
- index: Index operations (start, status, clear)
- search: Unified search for code or memory
- validate: Validation and analysis operations
- memory: Memory storage, retrieval, timeline, inject
- session: Agent session lifecycle + summaries
- agent: Agent activity logging
- project: Project workflow management
- vcs: Repository operations
"#
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
