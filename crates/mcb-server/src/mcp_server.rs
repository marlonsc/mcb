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
    ContextServiceInterface, IndexingServiceInterface, MemoryServiceInterface,
    SearchServiceInterface, ValidationServiceInterface,
};
use mcb_domain::ports::providers::VcsProvider;

use crate::handlers::{
    AnalyzeComplexityHandler, AnalyzeImpactHandler, ClearIndexHandler, CompareBranchesHandler,
    CreateSessionSummaryHandler, GetIndexingStatusHandler, GetSessionSummaryHandler,
    GetValidationRulesHandler, IndexCodebaseHandler, IndexVcsRepositoryHandler,
    ListRepositoriesHandler, ListValidatorsHandler, MemoryGetObservationsHandler,
    MemoryInjectContextHandler, MemorySearchHandler, MemoryTimelineHandler, SearchBranchHandler,
    SearchCodeHandler, SearchMemoriesHandler, StoreObservationHandler, ValidateArchitectureHandler,
    ValidateFileHandler,
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
    memory: Arc<dyn MemoryServiceInterface>,
    #[allow(dead_code)] // Reserved for future branch/compare tool wiring
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
        vcs_provider: Arc<dyn VcsProvider>,
    ) -> Self {
        let handlers = ToolHandlers {
            index_codebase: Arc::new(IndexCodebaseHandler::new(indexing_service.clone())),
            search_code: Arc::new(SearchCodeHandler::new(search_service.clone())),
            get_indexing_status: Arc::new(GetIndexingStatusHandler::new(indexing_service.clone())),
            clear_index: Arc::new(ClearIndexHandler::new(indexing_service.clone())),
            validate_architecture: Arc::new(ValidateArchitectureHandler::new(
                validation_service.clone(),
            )),
            validate_file: Arc::new(ValidateFileHandler::new(validation_service.clone())),
            list_validators: Arc::new(ListValidatorsHandler::new(validation_service.clone())),
            get_validation_rules: Arc::new(GetValidationRulesHandler::new(
                validation_service.clone(),
            )),
            analyze_complexity: Arc::new(AnalyzeComplexityHandler::new(validation_service)),
            index_vcs_repository: Arc::new(IndexVcsRepositoryHandler::new(vcs_provider.clone())),
            search_branch: Arc::new(SearchBranchHandler::new(vcs_provider.clone())),
            list_repositories: Arc::new(ListRepositoriesHandler::new()),
            compare_branches: Arc::new(CompareBranchesHandler::new(vcs_provider.clone())),
            analyze_impact: Arc::new(AnalyzeImpactHandler::new(vcs_provider.clone())),
            store_observation: Arc::new(StoreObservationHandler::new(memory_service.clone())),
            search_memories: Arc::new(SearchMemoriesHandler::new(memory_service.clone())),
            get_session_summary: Arc::new(GetSessionSummaryHandler::new(memory_service.clone())),
            create_session_summary: Arc::new(CreateSessionSummaryHandler::new(
                memory_service.clone(),
            )),
            memory_timeline: Arc::new(MemoryTimelineHandler::new(memory_service.clone())),
            memory_get_observations: Arc::new(MemoryGetObservationsHandler::new(
                memory_service.clone(),
            )),
            memory_inject_context: Arc::new(MemoryInjectContextHandler::new(
                memory_service.clone(),
            )),
            memory_search: Arc::new(MemorySearchHandler::new(memory_service.clone())),
        };

        Self {
            services: McpServices {
                indexing: indexing_service,
                context: context_service,
                search: search_service,
                memory: memory_service,
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

    /// Access to search service
    pub fn search_service(&self) -> Arc<dyn SearchServiceInterface> {
        Arc::clone(&self.services.search)
    }

    /// Access to memory service
    pub fn memory_service(&self) -> Arc<dyn MemoryServiceInterface> {
        Arc::clone(&self.services.memory)
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

    /// Access to validate file handler (for HTTP transport)
    pub fn validate_file_handler(&self) -> Arc<ValidateFileHandler> {
        Arc::clone(&self.handlers.validate_file)
    }

    /// Access to list validators handler (for HTTP transport)
    pub fn list_validators_handler(&self) -> Arc<ListValidatorsHandler> {
        Arc::clone(&self.handlers.list_validators)
    }

    /// Access to get validation rules handler (for HTTP transport)
    pub fn get_validation_rules_handler(&self) -> Arc<GetValidationRulesHandler> {
        Arc::clone(&self.handlers.get_validation_rules)
    }

    /// Access to analyze complexity handler (for HTTP transport)
    pub fn analyze_complexity_handler(&self) -> Arc<AnalyzeComplexityHandler> {
        Arc::clone(&self.handlers.analyze_complexity)
    }

    /// Access to index VCS repository handler (for HTTP transport)
    pub fn index_vcs_repository_handler(&self) -> Arc<IndexVcsRepositoryHandler> {
        Arc::clone(&self.handlers.index_vcs_repository)
    }

    /// Access to search branch handler (for HTTP transport)
    pub fn search_branch_handler(&self) -> Arc<SearchBranchHandler> {
        Arc::clone(&self.handlers.search_branch)
    }

    /// Access to list repositories handler (for HTTP transport)
    pub fn list_repositories_handler(&self) -> Arc<ListRepositoriesHandler> {
        Arc::clone(&self.handlers.list_repositories)
    }

    /// Access to compare branches handler (for HTTP transport)
    pub fn compare_branches_handler(&self) -> Arc<CompareBranchesHandler> {
        Arc::clone(&self.handlers.compare_branches)
    }

    /// Access to analyze impact handler (for HTTP transport)
    pub fn analyze_impact_handler(&self) -> Arc<AnalyzeImpactHandler> {
        Arc::clone(&self.handlers.analyze_impact)
    }

    /// Access to store observation handler (for HTTP transport)
    pub fn store_observation_handler(&self) -> Arc<StoreObservationHandler> {
        Arc::clone(&self.handlers.store_observation)
    }

    /// Access to search memories handler (for HTTP transport)
    pub fn search_memories_handler(&self) -> Arc<SearchMemoriesHandler> {
        Arc::clone(&self.handlers.search_memories)
    }

    /// Access to get session summary handler (for HTTP transport)
    pub fn get_session_summary_handler(&self) -> Arc<GetSessionSummaryHandler> {
        Arc::clone(&self.handlers.get_session_summary)
    }

    /// Access to create session summary handler (for HTTP transport)
    pub fn create_session_summary_handler(&self) -> Arc<CreateSessionSummaryHandler> {
        Arc::clone(&self.handlers.create_session_summary)
    }

    /// Access to memory timeline handler (for HTTP transport)
    pub fn memory_timeline_handler(&self) -> Arc<MemoryTimelineHandler> {
        Arc::clone(&self.handlers.memory_timeline)
    }

    /// Access to memory get observations handler (for HTTP transport)
    pub fn memory_get_observations_handler(&self) -> Arc<MemoryGetObservationsHandler> {
        Arc::clone(&self.handlers.memory_get_observations)
    }

    /// Access to memory inject context handler (for HTTP transport)
    pub fn memory_inject_context_handler(&self) -> Arc<MemoryInjectContextHandler> {
        Arc::clone(&self.handlers.memory_inject_context)
    }

    /// Access to memory search handler (for HTTP transport)
    pub fn memory_search_handler(&self) -> Arc<MemorySearchHandler> {
        Arc::clone(&self.handlers.memory_search)
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
                 - validate_architecture: Run architecture validation rules on a codebase\n\
                 - validate_file: Validate a single file against architecture rules\n\
                 - list_validators: List available validators\n\
                 - get_validation_rules: Get validation rules by category\n\
                 - analyze_complexity: Get code complexity metrics for a file\n\
                 - store_observation: Store an observation in the semantic memory\n\
                 - search_memories: Search observations in semantic memory using a natural language query\n\
                 - get_session_summary: Retrieve a summary for a specific session ID\n\
                 - create_session_summary: Create or update a summary for a coding session\n\
                 "
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
