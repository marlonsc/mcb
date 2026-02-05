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

use mcb_application::ports::services::AgentSessionServiceInterface;
use mcb_application::{
    ContextServiceInterface, IndexingServiceInterface, MemoryServiceInterface,
    SearchServiceInterface, ValidationServiceInterface,
};
use mcb_domain::ports::providers::VcsProvider;

use crate::handlers::{
    AgentHandler, IndexHandler, MemoryHandler, ProjectHandler, SearchHandler, SessionHandler,
    ValidateHandler, VcsHandler,
};
use crate::hooks::HookProcessor;
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
    memory: Arc<dyn MemoryServiceInterface>,
    agent_session: Arc<dyn AgentSessionServiceInterface>,
    vcs: Arc<dyn VcsProvider>,
}

impl McpServer {
    /// Create a new MCP server with injected dependencies
    pub fn new(
        indexing_service: Arc<dyn IndexingServiceInterface>,
        context_service: Arc<dyn ContextServiceInterface>,
        search_service: Arc<dyn SearchServiceInterface>,
        validation_service: Arc<dyn ValidationServiceInterface>,
        memory_service: Arc<dyn MemoryServiceInterface>,
        agent_session_service: Arc<dyn AgentSessionServiceInterface>,
        vcs_provider: Arc<dyn VcsProvider>,
    ) -> Self {
        let hook_processor = HookProcessor::new(Some(memory_service.clone()));

        let handlers = ToolHandlers {
            index: Arc::new(IndexHandler::new(indexing_service.clone())),
            search: Arc::new(SearchHandler::new(
                search_service.clone(),
                memory_service.clone(),
            )),
            validate: Arc::new(ValidateHandler::new(validation_service)),
            memory: Arc::new(MemoryHandler::new(memory_service.clone())),
            session: Arc::new(SessionHandler::new(
                agent_session_service.clone(),
                memory_service.clone(),
            )),
            agent: Arc::new(AgentHandler::new(agent_session_service.clone())),
            project: Arc::new(ProjectHandler),
            vcs: Arc::new(VcsHandler::new(vcs_provider.clone())),
            hook_processor: Arc::new(hook_processor),
        };

        Self {
            services: McpServices {
                indexing: indexing_service,
                context: context_service,
                search: search_service,
                memory: memory_service,
                agent_session: agent_session_service,
                vcs: vcs_provider,
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

    /// Access to VCS provider (for branch/repo handlers)
    pub fn vcs_provider(&self) -> Arc<dyn VcsProvider> {
        Arc::clone(&self.services.vcs)
    }

    /// Access to search service
    pub fn search_service(&self) -> Arc<dyn SearchServiceInterface> {
        Arc::clone(&self.services.search)
    }

    /// Access to memory service
    pub fn memory_service(&self) -> Arc<dyn MemoryServiceInterface> {
        Arc::clone(&self.services.memory)
    }

    /// Access to agent session service
    pub fn agent_session_service(&self) -> Arc<dyn AgentSessionServiceInterface> {
        Arc::clone(&self.services.agent_session)
    }

    /// Access to consolidated index handler (for HTTP transport)
    pub fn index_handler(&self) -> Arc<IndexHandler> {
        Arc::clone(&self.handlers.index)
    }

    /// Access to consolidated search handler (for HTTP transport)
    pub fn search_handler(&self) -> Arc<SearchHandler> {
        Arc::clone(&self.handlers.search)
    }

    /// Access to consolidated validate handler (for HTTP transport)
    pub fn validate_handler(&self) -> Arc<ValidateHandler> {
        Arc::clone(&self.handlers.validate)
    }

    /// Access to consolidated memory handler (for HTTP transport)
    pub fn memory_handler(&self) -> Arc<MemoryHandler> {
        Arc::clone(&self.handlers.memory)
    }

    /// Access to consolidated session handler (for HTTP transport)
    pub fn session_handler(&self) -> Arc<SessionHandler> {
        Arc::clone(&self.handlers.session)
    }

    /// Access to consolidated agent handler (for HTTP transport)
    pub fn agent_handler(&self) -> Arc<AgentHandler> {
        Arc::clone(&self.handlers.agent)
    }

    /// Access to consolidated project handler (for HTTP transport)
    pub fn project_handler(&self) -> Arc<ProjectHandler> {
        Arc::clone(&self.handlers.project)
    }

    /// Access to consolidated VCS handler (for HTTP transport)
    pub fn vcs_handler(&self) -> Arc<VcsHandler> {
        Arc::clone(&self.handlers.vcs)
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
                "MCP Context Browser - Semantic Code Search\n\n\
                 Consolidated tools:\n\
                 - index: Index operations (start, status, clear)\n\
                 - search: Unified search for code or memory\n\
                 - validate: Validation and analysis operations\n\
                 - memory: Memory storage, retrieval, timeline, inject\n\
                 - session: Agent session lifecycle + summaries\n\
                 - agent: Agent activity logging\n\
                 - project: Project workflow operations\n\
                 - vcs: Repository operations\n"
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
