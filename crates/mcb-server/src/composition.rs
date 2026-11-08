//! MCP server composition from a resolution context.
//!
//! Single composition root used by the Loco initializer and tests.
//! All handler wiring goes through Loco; this function builds [`McpServerBootstrap`]
//! from decomposed DI parts (database, providers, registry context).
//!
//! ## Pure Registry DI (ADR-050 + ADR-053)
//!
//! All services are resolved via the linkme registry. Shared providers (embedding,
//! vector store) are pre-resolved at startup and passed as parameters.
use std::sync::Arc;

use mcb_domain::ports::{
    EmbeddingProvider, HybridSearchProvider, IndexingOperationsInterface,
    ValidationOperationsInterface, VectorStoreProvider,
};
use mcb_domain::registry::admin_operations::{
    IndexingOperationsProviderConfig, ValidationOperationsProviderConfig,
    resolve_indexing_operations_provider, resolve_validation_operations_provider,
};
use mcb_domain::registry::database::{DatabaseRepositories, resolve_database_repositories};
use mcb_domain::registry::project_detection::{
    ProjectDetectionServiceConfig, resolve_project_detection_service,
};
use mcb_domain::registry::services::{
    resolve_agent_session_service, resolve_context_service, resolve_indexing_service,
    resolve_memory_service, resolve_search_service, resolve_validation_service,
};
use mcb_domain::registry::vcs::{VcsProviderConfig, resolve_vcs_provider};

use crate::handlers::{
    AgentHandler, EntityHandler, IndexHandler, IssueEntityHandler, MemoryHandler, OrgEntityHandler,
    PlanEntityHandler, ProjectHandler, SearchHandler, SessionHandler, ValidateHandler,
    VcsEntityHandler, VcsHandler,
};
use crate::hooks::HookProcessor;
use crate::mcp_server::{McpEntityRepositories, McpServer, McpServices};
use crate::state::McpServerBootstrap;
use crate::tools::{ExecutionFlow, ToolHandlers};
use mcb_utils::constants::{
    DEFAULT_DATABASE_PROVIDER, DEFAULT_LANGUAGE_PROVIDER, DEFAULT_NAMESPACE, DEFAULT_VCS_PROVIDER,
};

/// Shared providers resolved before MCP server composition.
pub struct McpBootstrapProviders {
    /// Shared embedding provider resolved at startup.
    pub embedding: Arc<dyn EmbeddingProvider>,
    /// Shared vector store provider resolved at startup.
    pub vector_store: Arc<dyn VectorStoreProvider>,
    /// Hybrid search provider for combined BM25 and semantic search.
    pub hybrid_search: Arc<dyn HybridSearchProvider>,
    /// Execution flow selected from runtime MCP configuration.
    pub execution_flow: ExecutionFlow,
}

/// Build MCP server and dashboard/auth ports from decomposed DI parts.
///
/// Uses **pure registry DI** (ADR-050 + ADR-053): shared providers are pre-resolved
/// at startup and passed in. All services are built via linkme registry resolution.
/// Zero direct `::new()` construction of infrastructure services.
///
/// # Arguments
///
/// * `registry_ctx` - Opaque context for linkme service registry resolution (downcast internally).
/// * `db_connection` - Database connection boxed as `Any` for registry database resolution.
/// * `providers` - Shared providers and execution flow resolved at startup.
///
/// # Errors
///
/// Returns a domain error if any service or repository resolution fails.
pub fn build_mcp_server_bootstrap(
    registry_ctx: &dyn std::any::Any,
    db_connection: Arc<dyn std::any::Any + Send + Sync>,
    providers: McpBootstrapProviders,
) -> mcb_domain::Result<McpServerBootstrap> {
    // 1. Resolve DB repos
    let repos = resolve_database_repositories(
        DEFAULT_DATABASE_PROVIDER,
        db_connection,
        DEFAULT_NAMESPACE.to_owned(),
    )?;

    // 2. Create shared operation trackers for admin endpoints
    let (indexing_ops, validation_ops) = resolve_admin_operation_trackers()?;

    // 3. Build MCP services struct from registry-resolved services
    let mcp_services = build_mcp_services(registry_ctx, &repos, providers.hybrid_search)?;

    let vcs_for_defaults = Arc::clone(&mcp_services.vcs);
    let handlers = build_tool_handlers(&mcp_services);
    let mcp_server = Arc::new(McpServer::new(
        mcp_services,
        handlers,
        &vcs_for_defaults,
        Some(providers.execution_flow),
    ));

    // 5. Build bootstrap with shared ports from context
    Ok(McpServerBootstrap {
        mcp_server,
        dashboard: repos.dashboard,
        auth_repo: repos.auth,
        embedding_provider: providers.embedding,
        vector_store: providers.vector_store,
        indexing_ops,
        validation_ops,
    })
}

/// Resolve the shared indexing/validation operation trackers used by admin endpoints.
fn resolve_admin_operation_trackers() -> mcb_domain::Result<(
    Arc<dyn IndexingOperationsInterface>,
    Arc<dyn ValidationOperationsInterface>,
)> {
    let indexing_ops = resolve_indexing_operations_provider(
        &IndexingOperationsProviderConfig::new(mcb_utils::constants::DEFAULT_INDEXING_OP_PROVIDER),
    )?;
    let validation_ops =
        resolve_validation_operations_provider(&ValidationOperationsProviderConfig::new(
            mcb_utils::constants::DEFAULT_VALIDATION_OP_PROVIDER,
        ))?;
    Ok((indexing_ops, validation_ops))
}

/// Resolve all MCP services via the linkme registry and assemble [`McpServices`].
fn build_mcp_services(
    registry_ctx: &dyn std::any::Any,
    repos: &DatabaseRepositories,
    hybrid_search: Arc<dyn HybridSearchProvider>,
) -> mcb_domain::Result<McpServices> {
    Ok(McpServices {
        indexing: resolve_indexing_service(registry_ctx)?,
        context: resolve_context_service(registry_ctx)?,
        search: resolve_search_service(registry_ctx)?,
        validation: resolve_validation_service(registry_ctx)?,
        memory: resolve_memory_service(registry_ctx)?,
        agent_session: resolve_agent_session_service(registry_ctx)?,
        project: resolve_project_detection_service(&ProjectDetectionServiceConfig::new(
            DEFAULT_LANGUAGE_PROVIDER,
        ))?,
        project_workflow: Arc::clone(&repos.project),
        vcs: resolve_vcs_provider(&VcsProviderConfig::new(DEFAULT_VCS_PROVIDER))?,
        hybrid_search,
        entities: McpEntityRepositories {
            vcs: Arc::clone(&repos.vcs_entity),
            plan: Arc::clone(&repos.plan_entity),
            issue: Arc::clone(&repos.issue_entity),
            org: Arc::clone(&repos.org_entity),
        },
    })
}

/// Build the full set of tool handlers from resolved services.
///
/// This is the single composition point for handler wiring: every handler
/// `Arc` is constructed exactly here, receiving already-built dependencies.
fn build_tool_handlers(services: &McpServices) -> ToolHandlers {
    let hook_processor = HookProcessor::new(Some(Arc::clone(&services.memory)));
    let vcs_entity_handler = Arc::new(VcsEntityHandler::new(Arc::clone(&services.entities.vcs)));
    let plan_entity_handler = Arc::new(PlanEntityHandler::new(Arc::clone(&services.entities.plan)));
    let issue_entity_handler = Arc::new(IssueEntityHandler::new(Arc::clone(
        &services.entities.issue,
    )));
    let org_entity_handler = Arc::new(OrgEntityHandler::new(Arc::clone(&services.entities.org)));
    let entity_handler = Arc::new(EntityHandler::new(
        Arc::clone(&vcs_entity_handler),
        Arc::clone(&plan_entity_handler),
        Arc::clone(&issue_entity_handler),
        Arc::clone(&org_entity_handler),
    ));

    ToolHandlers {
        index: Arc::new(IndexHandler::new(Arc::clone(&services.indexing))),
        search: Arc::new(SearchHandler::new(
            Arc::clone(&services.search),
            Arc::clone(&services.memory),
            Arc::clone(&services.hybrid_search),
            Arc::clone(&services.indexing),
        )),
        validate: Arc::new(ValidateHandler::new(Arc::clone(&services.validation))),
        memory: Arc::new(MemoryHandler::new(
            Arc::clone(&services.memory),
            mcb_utils::utils::vcs_context::capture_vcs_context(),
        )),
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
    }
}
