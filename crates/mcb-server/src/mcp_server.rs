//! MCP server core.
//!
//! **Documentation**: [docs/modules/server.md](../../../docs/modules/server.md)
//!
//! [`McpServer`] orchestrates MCP tool execution and wires service dependencies
//! through [`McpServices`] and [`McpEntityRepositories`].

use std::path::Path;
use std::sync::Arc;

use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus, AgentType};
use mcb_domain::entities::project::Project;
use mcb_domain::ports::AgentSessionServiceInterface;
use mcb_domain::ports::HybridSearchProvider;
use mcb_domain::ports::VcsProvider;
use mcb_domain::ports::{
    ContextServiceInterface, IndexingServiceInterface, MemoryServiceInterface,
    ProjectDetectorService, SearchServiceInterface, ValidationServiceInterface,
};
use mcb_domain::ports::{
    IssueEntityRepository, OrgEntityRepository, PlanEntityRepository, ProjectRepository,
    VcsEntityRepository,
};
use rmcp::ErrorData as McpError;
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolResult, Implementation, ListToolsResult, PaginatedRequestParams, ProtocolVersion,
    ServerCapabilities, ServerInfo,
};
use tokio::sync::OnceCell;

use crate::handlers::{
    AgentHandler, EntityHandler, IndexHandler, IssueEntityHandler, MemoryHandler, OrgEntityHandler,
    PlanEntityHandler, ProjectHandler, SearchHandler, SessionHandler, ValidateHandler,
    VcsEntityHandler, VcsHandler,
};
use crate::hooks::HookProcessor;
use crate::tools::{
    ExecutionFlow, RuntimeDefaults, ToolExecutionContext, ToolHandlers, create_tool_list,
    route_tool_call,
};

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
    runtime_defaults: RuntimeDefaults,
    /// Lazy auto-initialization gate for session + project creation (T10, T11).
    auto_init: Arc<OnceCell<()>>,
}

impl std::fmt::Debug for McpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpServer").finish()
    }
}

/// Entity repositories used by MCP entity handlers.
#[derive(Clone)]
pub struct McpEntityRepositories {
    /// VCS entity repository (repos, branches, worktrees)
    pub vcs: Arc<dyn VcsEntityRepository>,
    /// Plan entity repository (plans, versions, reviews)
    pub plan: Arc<dyn PlanEntityRepository>,
    /// Issue entity repository (issues, comments, labels, assignments)
    pub issue: Arc<dyn IssueEntityRepository>,
    /// Org entity repository (orgs, users, teams, members, api keys)
    pub org: Arc<dyn OrgEntityRepository>,
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
    /// Project workflow repository
    pub project_workflow: Arc<dyn ProjectRepository>,
    /// VCS provider
    pub vcs: Arc<dyn VcsProvider>,
    /// Hybrid search provider for BM25+semantic re-ranking.
    pub hybrid_search: Arc<dyn HybridSearchProvider>,
    /// Entity repositories shared by CRUD handlers.
    pub entities: McpEntityRepositories,
}

impl McpServer {
    /// Create a new MCP server with injected dependencies
    #[must_use]
    pub fn new(
        services: McpServices,
        vcs: &Arc<dyn VcsProvider>,
        execution_flow: Option<ExecutionFlow>,
    ) -> Self {
        let runtime_defaults =
            futures::executor::block_on(RuntimeDefaults::discover(vcs.as_ref(), execution_flow));
        let hook_processor = HookProcessor::new(Some(Arc::clone(&services.memory)));
        let vcs_entity_handler =
            Arc::new(VcsEntityHandler::new(Arc::clone(&services.entities.vcs)));
        let plan_entity_handler =
            Arc::new(PlanEntityHandler::new(Arc::clone(&services.entities.plan)));
        let issue_entity_handler = Arc::new(IssueEntityHandler::new(Arc::clone(
            &services.entities.issue,
        )));
        let org_entity_handler =
            Arc::new(OrgEntityHandler::new(Arc::clone(&services.entities.org)));
        let entity_handler = Arc::new(EntityHandler::new(
            Arc::clone(&vcs_entity_handler),
            Arc::clone(&plan_entity_handler),
            Arc::clone(&issue_entity_handler),
            Arc::clone(&org_entity_handler),
        ));

        let handlers = ToolHandlers {
            index: Arc::new(IndexHandler::new(Arc::clone(&services.indexing))),
            search: Arc::new(SearchHandler::new(
                Arc::clone(&services.search),
                Arc::clone(&services.memory),
                Arc::clone(&services.hybrid_search),
                Arc::clone(&services.indexing),
            )),
            validate: Arc::new(ValidateHandler::new(Arc::clone(&services.validation))),
            memory: Arc::new(MemoryHandler::new(Arc::clone(&services.memory))),
            session: Arc::new(SessionHandler::new(
                Arc::clone(&services.agent_session),
                Arc::clone(&services.memory),
            )),
            agent: Arc::new(AgentHandler::new(Arc::clone(&services.agent_session))),
            project: Arc::new(ProjectHandler::new(Arc::clone(&services.project_workflow))),
            vcs: Arc::new(VcsHandler::new(Arc::clone(&services.vcs))),
            vcs_entity: vcs_entity_handler,
            plan_entity: plan_entity_handler,
            issue_entity: issue_entity_handler,
            org_entity: org_entity_handler,
            entity: entity_handler,
            hook_processor: Arc::new(hook_processor),
        };

        Self {
            services,
            handlers,
            runtime_defaults,
            auto_init: Arc::new(OnceCell::new()),
        }
    }

    impl_arc_accessors! {
        /// Access to indexing service
        indexing_service -> dyn IndexingServiceInterface => services.indexing,
        /// Access to context service
        context_service -> dyn ContextServiceInterface => services.context,
        /// Access to search service
        search_service -> dyn SearchServiceInterface => services.search,
        /// Access to validation service
        validation_service -> dyn ValidationServiceInterface => services.validation,
        /// Access to memory service
        memory_service -> dyn MemoryServiceInterface => services.memory,
        /// Access to agent session service
        agent_session_service -> dyn AgentSessionServiceInterface => services.agent_session,
        /// Access to project service
        project_service -> dyn ProjectDetectorService => services.project,
        /// Access to project workflow repository
        project_workflow_repository -> dyn ProjectRepository => services.project_workflow,
        /// Access to VCS provider
        vcs_provider -> dyn VcsProvider => services.vcs,
        /// Access to VCS entity repository
        vcs_entity_repository -> dyn VcsEntityRepository => services.entities.vcs,
        /// Access to plan entity repository
        plan_entity_repository -> dyn PlanEntityRepository => services.entities.plan,
        /// Access to issue entity repository
        issue_entity_repository -> dyn IssueEntityRepository => services.entities.issue,
        /// Access to organization entity repository
        org_entity_repository -> dyn OrgEntityRepository => services.entities.org,
        /// Access to index handler (for HTTP transport)
        index_handler -> IndexHandler => handlers.index,
        /// Access to search handler (for HTTP transport)
        search_handler -> SearchHandler => handlers.search,
        /// Access to validate handler (for HTTP transport)
        validate_handler -> ValidateHandler => handlers.validate,
        /// Access to memory handler (for HTTP transport)
        memory_handler -> MemoryHandler => handlers.memory,
        /// Access to session handler (for HTTP transport)
        session_handler -> SessionHandler => handlers.session,
        /// Access to agent handler (for HTTP transport)
        agent_handler -> AgentHandler => handlers.agent,
        /// Access to VCS handler (for HTTP transport)
        vcs_handler -> VcsHandler => handlers.vcs,
        /// Access to unified entity handler (for HTTP transport)
        entity_handler -> EntityHandler => handlers.entity,
        /// Access to project handler (for HTTP transport)
        project_handler -> ProjectHandler => handlers.project,
        /// Access to VCS entity handler (for HTTP transport)
        vcs_entity_handler -> VcsEntityHandler => handlers.vcs_entity,
        /// Access to plan entity handler (for HTTP transport)
        plan_entity_handler -> PlanEntityHandler => handlers.plan_entity,
        /// Access to issue entity handler (for HTTP transport)
        issue_entity_handler -> IssueEntityHandler => handlers.issue_entity,
        /// Access to org entity handler (for HTTP transport)
        org_entity_handler -> OrgEntityHandler => handlers.org_entity,
        /// Access to hook processor (for automatic memory operations)
        hook_processor -> HookProcessor => handlers.hook_processor,
    }

    /// Clone the complete tool handlers set for unified internal execution.
    #[must_use]
    pub fn tool_handlers(&self) -> ToolHandlers {
        self.handlers.clone()
    }

    /// Returns the runtime defaults discovered during server initialization.
    ///
    /// These defaults include workspace root, repository information, operator ID,
    /// machine ID, session ID, agent program, model ID, and execution flow.
    #[must_use]
    pub fn runtime_defaults(&self) -> RuntimeDefaults {
        self.runtime_defaults.clone()
    }
}

impl ServerHandler for McpServer {
    /// Get server information and capabilities
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "MCP Context Browser".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
                ..Default::default()
            },
            instructions: Some(
                "MCP Context Browser - Semantic Code Search

tools:
- index: Index operations (start, status, clear)
- search: Unified search for code or memory
- validate: Validation and analysis operations
- memory: Memory storage, retrieval, timeline, inject
- session: Agent session lifecycle + summaries
- agent: Agent activity logging
- project: Project workflow management
- vcs: Repository operations
- entity: Unified entity CRUD (vcs/plan/issue/org resources)
"
                .to_owned(),
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
        mut request: rmcp::model::CallToolRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let mut overrides = std::collections::HashMap::new();
        let extract_meta =
            |meta: Option<&rmcp::model::Meta>,
             map: &mut std::collections::HashMap<String, String>| {
                if let Some(m) = meta {
                    for (key, value) in m.iter() {
                        if let Some(string_value) = value.as_str() {
                            map.insert(key.clone(), string_value.to_owned());
                        } else if let Some(bool_value) = value.as_bool() {
                            map.insert(key.clone(), bool_value.to_string());
                        } else if let Some(number_value) = value.as_number() {
                            map.insert(key.clone(), number_value.to_string());
                        }
                    }
                }
            };

        extract_meta(Some(&context.meta), &mut overrides);
        extract_meta(request.meta.as_ref(), &mut overrides);

        let mut execution_context =
            ToolExecutionContext::resolve(&self.runtime_defaults, &overrides);

        if let Some(path_str) = execution_context.repo_path.as_deref()
            && execution_context
                .repo_id
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            && let Ok(repo) = self.services.vcs.open_repository(Path::new(path_str)).await
        {
            execution_context.repo_id = Some(self.services.vcs.repository_id(&repo).into_string());
        }

        execution_context.apply_to_request_if_missing(&mut request);

        // T10 + T11: Lazy auto-creation of session and project on first tool call.
        {
            let services = self.services.clone();
            let defaults = self.runtime_defaults.clone();
            let ctx = execution_context.clone();
            self.auto_init
                .get_or_init(|| async move {
                    auto_create_session_and_project(&services, &defaults, &ctx).await;
                })
                .await;
        }

        route_tool_call(request, &self.handlers, execution_context).await
    }
}

/// Auto-create agent session and project on first tool call (T10 + T11).
///
/// Called lazily via `OnceCell` so it runs exactly once per server boot and
/// does not block startup. Failures are non-fatal: logged as warnings but
/// never propagated to tool callers.
async fn auto_create_session_and_project(
    services: &McpServices,
    defaults: &RuntimeDefaults,
    ctx: &ToolExecutionContext,
) {
    // T10: Auto-create agent session with IDE identity
    if let Some(ref session_id) = ctx.session_id {
        let now = mcb_domain::utils::time::epoch_secs_i64().unwrap_or(0);
        let ide_label = defaults
            .agent_program
            .as_deref()
            .or(ctx.agent_program.as_deref())
            .unwrap_or("mcb-stdio");
        let session = AgentSession {
            id: session_id.clone(),
            session_summary_id: format!("auto_{}", mcb_domain::utils::id::generate().simple()),
            agent_type: AgentType::Sisyphus,
            model: ctx.model_id.clone().unwrap_or_else(|| "unknown".to_owned()),
            parent_session_id: ctx.parent_session_id.clone(),
            started_at: now,
            ended_at: None,
            duration_ms: None,
            status: AgentSessionStatus::Active,
            prompt_summary: Some(format!("Auto-session via {ide_label}")),
            result_summary: None,
            token_count: None,
            tool_calls_count: None,
            delegations_count: None,
            project_id: ctx.project_id.clone(),
            worktree_id: ctx.worktree_id.clone(),
        };
        match services.agent_session.create_session(session).await {
            Ok(_) => tracing::info!("Auto-session created: {session_id} via {ide_label}"),
            Err(e) => tracing::warn!("Auto-session creation failed (non-fatal): {e}"),
        }
    }

    // T11: Auto-create project from VCS context
    if let (Some(org_id), Some(repo_path)) = (&ctx.org_id, &ctx.repo_path) {
        let project_name = Path::new(repo_path.as_str())
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        match services
            .project_workflow
            .get_by_name(org_id, project_name)
            .await
        {
            Ok(_) => {
                tracing::debug!("Project '{project_name}' already exists for org '{org_id}'");
            }
            Err(_) => {
                let now = mcb_domain::utils::time::epoch_secs_i64().unwrap_or(0);
                let project = Project {
                    id: mcb_domain::utils::id::generate().to_string(),
                    org_id: org_id.clone(),
                    name: project_name.to_owned(),
                    path: repo_path.clone(),
                    created_at: now,
                    updated_at: now,
                };
                match services.project_workflow.create(&project).await {
                    Ok(()) => {
                        tracing::info!("Auto-project created: '{project_name}' for org '{org_id}'");
                    }
                    Err(e) => {
                        tracing::warn!("Auto-project creation failed (non-fatal): {e}");
                    }
                }
            }
        }
    }
}
