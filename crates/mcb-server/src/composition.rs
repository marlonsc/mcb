//! MCP server composition from a resolution context.
//!
//! Single composition root used by the Loco initializer and tests.
//! All handler wiring goes through Loco; this function builds [`McpServerBootstrap`]
//! from a [`ServiceResolutionContext`] (Loco-provided or test).
//!
//! ## Pure Registry DI (ADR-050 + ADR-053)
//!
//! All services are resolved via the linkme registry. Shared providers (embedding,
//! vector store) are pre-resolved at startup and stored in `ServiceResolutionContext`.
use std::sync::Arc;

use mcb_domain::ports::{IndexingOperationsInterface, ValidationOperationsInterface};
use mcb_domain::registry::admin_operations::{
    IndexingOperationsProviderConfig, ValidationOperationsProviderConfig,
    resolve_indexing_operations_provider, resolve_validation_operations_provider,
};
use mcb_domain::registry::database::resolve_database_repositories;
use mcb_domain::registry::project_detection::{
    ProjectDetectionServiceConfig, resolve_project_detection_service,
};
use mcb_domain::registry::services::{
    resolve_agent_session_service, resolve_context_service, resolve_indexing_service,
    resolve_memory_service, resolve_search_service, resolve_validation_service,
};
use mcb_domain::registry::vcs::{VcsProviderConfig, resolve_vcs_provider};
use mcb_infrastructure::resolution_context::ServiceResolutionContext;

use crate::mcp_server::{McpEntityRepositories, McpServer, McpServices};
use crate::state::McpServerBootstrap;
use crate::tools::ExecutionFlow;
/// Registry provider name for `SeaORM` database repositories.
const DATABASE_PROVIDER: &str = "seaorm";

/// Default namespace for database repositories.
const DEFAULT_NAMESPACE: &str = "default";

/// Registry provider name for universal language chunking.
const LANGUAGE_PROVIDER: &str = "universal";

/// Registry provider name for Git VCS.
const VCS_PROVIDER: &str = "git";

/// Build MCP server and dashboard/auth ports from a resolution context.
///
/// Uses **pure registry DI** (ADR-050 + ADR-053): shared providers are pre-resolved
/// in [`ServiceResolutionContext`], and all services are built via linkme registry
/// resolution. Zero direct `::new()` construction of infrastructure services.
///
/// # Errors
///
/// Returns a domain error if any service or repository resolution fails.
pub fn build_mcp_server_bootstrap(
    resolution_ctx: &ServiceResolutionContext,
    execution_flow: ExecutionFlow,
) -> mcb_domain::Result<McpServerBootstrap> {
    let raw_ctx: &dyn std::any::Any = resolution_ctx;

    // 1. Resolve DB repos
    let repos = resolve_database_repositories(
        DATABASE_PROVIDER,
        Box::new(resolution_ctx.db.clone()),
        DEFAULT_NAMESPACE.to_owned(),
    )?;

    // 2. Create shared operation trackers for admin endpoints
    let indexing_ops: Arc<dyn IndexingOperationsInterface> =
        resolve_indexing_operations_provider(&IndexingOperationsProviderConfig::new("default"))?;
    let validation_ops: Arc<dyn ValidationOperationsInterface> =
        resolve_validation_operations_provider(&ValidationOperationsProviderConfig::new(
            "default",
        ))?;

    // 3. Resolve ALL services via registry (shared providers from ServiceResolutionContext)
    let context_service = resolve_context_service(raw_ctx)?;
    let search_service = resolve_search_service(raw_ctx)?;
    let indexing_service = resolve_indexing_service(raw_ctx)?;
    let memory_service = resolve_memory_service(raw_ctx)?;

    // 4. Build MCP services struct
    let mcp_services = McpServices {
        indexing: indexing_service,
        context: context_service,
        search: search_service,
        validation: resolve_validation_service(raw_ctx)?,
        memory: memory_service,
        agent_session: resolve_agent_session_service(raw_ctx)?,
        project: resolve_project_detection_service(&ProjectDetectionServiceConfig::new(
            LANGUAGE_PROVIDER,
        ))?,
        project_workflow: Arc::clone(&repos.project),
        vcs: resolve_vcs_provider(&VcsProviderConfig::new(VCS_PROVIDER))?,
        entities: McpEntityRepositories {
            vcs: Arc::clone(&repos.vcs_entity),
            plan: Arc::clone(&repos.plan_entity),
            issue: Arc::clone(&repos.issue_entity),
            org: Arc::clone(&repos.org_entity),
        },
    };

    let vcs_for_defaults = Arc::clone(&mcp_services.vcs);
    let mcp_server = Arc::new(McpServer::new(
        mcp_services,
        &vcs_for_defaults,
        Some(execution_flow),
    ));

    // 5. Build bootstrap with shared ports from context
    Ok(McpServerBootstrap {
        mcp_server,
        dashboard: repos.dashboard,
        auth_repo: repos.auth,
        embedding_provider: Arc::clone(&resolution_ctx.embedding_provider),
        vector_store: Arc::clone(&resolution_ctx.vector_store_provider),
        indexing_ops,
        validation_ops,
    })
}
