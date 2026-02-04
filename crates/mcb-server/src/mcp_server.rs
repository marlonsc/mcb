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

use mcb_application::ports::services::AgentSessionServiceInterface;
use mcb_application::{
    ContextServiceInterface, IndexingServiceInterface, MemoryServiceInterface,
    SearchServiceInterface, ValidationServiceInterface,
};
use mcb_domain::ports::providers::VcsProvider;

use crate::handlers::{
    AnalyzeComplexityHandler, AnalyzeImpactHandler, ClearIndexHandler, CompareBranchesHandler,
    CreateAgentSessionHandler, CreateSessionSummaryHandler, GetAgentSessionHandler,
    GetExecutionsHandler, GetIndexingStatusHandler, GetQualityGatesHandler,
    GetSessionSummaryHandler, GetValidationRulesHandler, IndexCodebaseHandler,
    IndexVcsRepositoryHandler, ListAgentSessionsHandler, ListRepositoriesHandler,
    ListValidatorsHandler, MemoryGetErrorPatternsHandler, MemoryGetObservationsHandler,
    MemoryInjectContextHandler, MemoryRecordErrorPatternHandler, MemorySearchHandler,
    MemoryTimelineHandler, ProjectAddDependencyHandler, ProjectCreateIssueHandler,
    ProjectCreatePhaseHandler, ProjectListDecisionsHandler, ProjectListIssuesHandler,
    ProjectListPhasesHandler, ProjectRecordDecisionHandler, ProjectUpdateIssueHandler,
    ProjectUpdatePhaseHandler, SearchBranchHandler, SearchCodeHandler, SearchMemoriesHandler,
    StoreDelegationHandler, StoreExecutionHandler, StoreObservationHandler,
    StoreQualityGateHandler, StoreToolCallHandler, UpdateAgentSessionHandler,
    ValidateArchitectureHandler, ValidateFileHandler,
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
            memory_store_execution: Arc::new(StoreExecutionHandler::new(memory_service.clone())),
            memory_get_executions: Arc::new(GetExecutionsHandler::new(memory_service.clone())),
            memory_store_quality_gate: Arc::new(StoreQualityGateHandler::new(
                memory_service.clone(),
            )),
            memory_get_quality_gates: Arc::new(GetQualityGatesHandler::new(memory_service.clone())),
            memory_record_error_pattern: Arc::new(MemoryRecordErrorPatternHandler::new(
                memory_service.clone(),
            )),
            memory_get_error_patterns: Arc::new(MemoryGetErrorPatternsHandler::new(
                memory_service.clone(),
            )),
            create_agent_session: Arc::new(CreateAgentSessionHandler::new(
                agent_session_service.clone(),
            )),
            get_agent_session: Arc::new(GetAgentSessionHandler::new(agent_session_service.clone())),
            update_agent_session: Arc::new(UpdateAgentSessionHandler::new(
                agent_session_service.clone(),
            )),
            list_agent_sessions: Arc::new(ListAgentSessionsHandler::new(
                agent_session_service.clone(),
            )),
            store_tool_call: Arc::new(StoreToolCallHandler::new(agent_session_service.clone())),
            store_delegation: Arc::new(StoreDelegationHandler::new(agent_session_service.clone())),
            project_create_phase: Arc::new(ProjectCreatePhaseHandler::new()),
            project_update_phase: Arc::new(ProjectUpdatePhaseHandler::new()),
            project_list_phases: Arc::new(ProjectListPhasesHandler::new()),
            project_create_issue: Arc::new(ProjectCreateIssueHandler::new()),
            project_update_issue: Arc::new(ProjectUpdateIssueHandler::new()),
            project_list_issues: Arc::new(ProjectListIssuesHandler::new()),
            project_add_dependency: Arc::new(ProjectAddDependencyHandler::new()),
            project_record_decision: Arc::new(ProjectRecordDecisionHandler::new()),
            project_list_decisions: Arc::new(ProjectListDecisionsHandler::new()),
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

    /// Access to memory store execution handler (for HTTP transport)
    pub fn memory_store_execution_handler(&self) -> Arc<StoreExecutionHandler> {
        Arc::clone(&self.handlers.memory_store_execution)
    }

    /// Access to memory get executions handler (for HTTP transport)
    pub fn memory_get_executions_handler(&self) -> Arc<GetExecutionsHandler> {
        Arc::clone(&self.handlers.memory_get_executions)
    }

    pub fn memory_store_quality_gate_handler(&self) -> Arc<StoreQualityGateHandler> {
        Arc::clone(&self.handlers.memory_store_quality_gate)
    }

    pub fn memory_get_quality_gates_handler(&self) -> Arc<GetQualityGatesHandler> {
        Arc::clone(&self.handlers.memory_get_quality_gates)
    }

    pub fn memory_record_error_pattern_handler(&self) -> Arc<MemoryRecordErrorPatternHandler> {
        Arc::clone(&self.handlers.memory_record_error_pattern)
    }

    pub fn memory_get_error_patterns_handler(&self) -> Arc<MemoryGetErrorPatternsHandler> {
        Arc::clone(&self.handlers.memory_get_error_patterns)
    }

    pub fn create_agent_session_handler(&self) -> Arc<CreateAgentSessionHandler> {
        Arc::clone(&self.handlers.create_agent_session)
    }

    pub fn get_agent_session_handler(&self) -> Arc<GetAgentSessionHandler> {
        Arc::clone(&self.handlers.get_agent_session)
    }

    pub fn update_agent_session_handler(&self) -> Arc<UpdateAgentSessionHandler> {
        Arc::clone(&self.handlers.update_agent_session)
    }

    pub fn list_agent_sessions_handler(&self) -> Arc<ListAgentSessionsHandler> {
        Arc::clone(&self.handlers.list_agent_sessions)
    }

    pub fn store_tool_call_handler(&self) -> Arc<StoreToolCallHandler> {
        Arc::clone(&self.handlers.store_tool_call)
    }

    pub fn store_delegation_handler(&self) -> Arc<StoreDelegationHandler> {
        Arc::clone(&self.handlers.store_delegation)
    }

    pub fn project_create_phase_handler(&self) -> Arc<ProjectCreatePhaseHandler> {
        Arc::clone(&self.handlers.project_create_phase)
    }

    pub fn project_update_phase_handler(&self) -> Arc<ProjectUpdatePhaseHandler> {
        Arc::clone(&self.handlers.project_update_phase)
    }

    pub fn project_list_phases_handler(&self) -> Arc<ProjectListPhasesHandler> {
        Arc::clone(&self.handlers.project_list_phases)
    }

    pub fn project_create_issue_handler(&self) -> Arc<ProjectCreateIssueHandler> {
        Arc::clone(&self.handlers.project_create_issue)
    }

    pub fn project_update_issue_handler(&self) -> Arc<ProjectUpdateIssueHandler> {
        Arc::clone(&self.handlers.project_update_issue)
    }

    pub fn project_list_issues_handler(&self) -> Arc<ProjectListIssuesHandler> {
        Arc::clone(&self.handlers.project_list_issues)
    }

    pub fn project_add_dependency_handler(&self) -> Arc<ProjectAddDependencyHandler> {
        Arc::clone(&self.handlers.project_add_dependency)
    }

    pub fn project_record_decision_handler(&self) -> Arc<ProjectRecordDecisionHandler> {
        Arc::clone(&self.handlers.project_record_decision)
    }

    pub fn project_list_decisions_handler(&self) -> Arc<ProjectListDecisionsHandler> {
        Arc::clone(&self.handlers.project_list_decisions)
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
