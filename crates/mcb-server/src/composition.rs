//! MCP server composition from a resolution context.
//!
//! Single composition root used by the Loco initializer and tests.
//! All handler wiring goes through Loco; this function builds [`McpServerBootstrap`]
//! from a [`ServiceResolutionContext`] (Loco-provided or test).
//!
//! ## Single-Resolution DI (ADR-050 SSOT)
//!
//! Providers are resolved **once** and shared across all services:
//! - `EmbeddingProvider` — single instance for context, memory, and health
//! - `VectorStoreProvider` — single instance for context, memory, and collections
//! - `IndexingOperationsInterface` — shared tracker for jobs admin
//! - `ValidationOperationsInterface` — shared tracker for jobs admin

use std::sync::Arc;

use mcb_domain::ports::{
    ContextServiceInterface, EmbeddingProvider, IndexingOperationsInterface,
    IndexingServiceInterface, MemoryServiceInterface, SearchServiceInterface,
    ValidationOperationsInterface, VectorStoreProvider,
};
use mcb_domain::registry::database::resolve_database_repositories;
use mcb_domain::registry::embedding::{EmbeddingProviderConfig, resolve_embedding_provider};
use mcb_domain::registry::language::{LanguageProviderConfig, resolve_language_provider};
use mcb_domain::registry::project_detection::{
    ProjectDetectionServiceConfig, resolve_project_detection_service,
};
use mcb_domain::registry::services::{resolve_agent_session_service, resolve_validation_service};
use mcb_domain::registry::vcs::{VcsProviderConfig, resolve_vcs_provider};
use mcb_domain::registry::vector_store::{
    VectorStoreProviderConfig, resolve_vector_store_provider,
};
use mcb_infrastructure::infrastructure::DefaultIndexingOperations;
use mcb_infrastructure::infrastructure::DefaultValidationOperations;
use mcb_infrastructure::resolution_context::ServiceResolutionContext;
use mcb_infrastructure::services::{
    ContextServiceImpl, IndexingServiceDeps, IndexingServiceImpl, IndexingServiceWithHashDeps,
    MemoryServiceImpl, SearchServiceImpl,
};

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
/// Uses **single-resolution DI**: embedding and vector store providers are resolved
/// once and injected into all services that need them. This ensures shared state
/// (health checks, operation tracking) is consistent across the application.
///
/// # Errors
///
/// Returns a domain error if any service or repository resolution fails.
pub fn build_mcp_server_bootstrap(
    resolution_ctx: &ServiceResolutionContext,
    execution_flow: ExecutionFlow,
) -> mcb_domain::Result<McpServerBootstrap> {
    let raw_ctx: &dyn std::any::Any = resolution_ctx;
    let config = &resolution_ctx.config;

    // 1. Resolve DB repos (unchanged)
    let repos = resolve_database_repositories(
        DATABASE_PROVIDER,
        Box::new(resolution_ctx.db.clone()),
        DEFAULT_NAMESPACE.to_owned(),
    )?;

    // 2. Resolve shared providers ONCE (single-resolution DI)
    let embedding_provider = resolve_embedding_from_config(config)?;
    let vector_store_provider = resolve_vector_store_from_config(config)?;

    // 3. Create shared operation trackers
    let indexing_ops: Arc<dyn IndexingOperationsInterface> =
        Arc::new(DefaultIndexingOperations::new());
    let validation_ops: Arc<dyn ValidationOperationsInterface> =
        Arc::new(DefaultValidationOperations::new());

    // 4. Build services with INJECTED shared providers
    let context_service: Arc<dyn ContextServiceInterface> = Arc::new(ContextServiceImpl::new(
        Arc::clone(&embedding_provider),
        Arc::clone(&vector_store_provider),
    ));

    let search_service: Arc<dyn SearchServiceInterface> =
        Arc::new(SearchServiceImpl::new(Arc::clone(&context_service)));

    let language_chunker =
        resolve_language_provider(&LanguageProviderConfig::new(LANGUAGE_PROVIDER))?;
    let indexing_service: Arc<dyn IndexingServiceInterface> = Arc::new(
        IndexingServiceImpl::new_with_file_hash_repository(IndexingServiceWithHashDeps {
            service: IndexingServiceDeps {
                context_service: Arc::clone(&context_service),
                language_chunker,
                indexing_ops: Arc::clone(&indexing_ops),
                event_bus: Arc::clone(&resolution_ctx.event_bus),
                supported_extensions: config.mcp.indexing.supported_extensions.clone(),
            },
            file_hash_repository: repos.file_hash,
        }),
    );

    let memory_service: Arc<dyn MemoryServiceInterface> = Arc::new(MemoryServiceImpl::new(
        DEFAULT_NAMESPACE.to_owned(),
        repos.memory,
        Arc::clone(&embedding_provider),
        Arc::clone(&vector_store_provider),
    ));

    // 5. Remaining services via registry (don't need shared providers)
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

    // 6. Build bootstrap with shared ports for admin controllers
    Ok(McpServerBootstrap {
        mcp_server,
        dashboard: repos.dashboard,
        auth_repo: repos.auth,
        embedding_provider,
        vector_store: vector_store_provider,
        indexing_ops,
        validation_ops,
    })
}

// ---------------------------------------------------------------------------
// Provider Config Helpers (extract AppConfig → registry config)
// ---------------------------------------------------------------------------

/// Build `EmbeddingProviderConfig` from application config and resolve the provider.
fn resolve_embedding_from_config(
    config: &mcb_infrastructure::config::AppConfig,
) -> mcb_domain::Result<Arc<dyn EmbeddingProvider>> {
    let mut embed_cfg = EmbeddingProviderConfig::new(
        config
            .providers
            .embedding
            .provider
            .as_deref()
            .unwrap_or("null"),
    );
    if let Some(ref v) = config.providers.embedding.cache_dir {
        embed_cfg = embed_cfg.with_cache_dir(v.clone());
    }
    if let Some(ref v) = config.providers.embedding.model {
        embed_cfg = embed_cfg.with_model(v.clone());
    }
    if let Some(ref v) = config.providers.embedding.base_url {
        embed_cfg = embed_cfg.with_base_url(v.clone());
    }
    if let Some(ref v) = config.providers.embedding.api_key {
        embed_cfg = embed_cfg.with_api_key(v.clone());
    }
    if let Some(d) = config.providers.embedding.dimensions {
        embed_cfg = embed_cfg.with_dimensions(d);
    }
    resolve_embedding_provider(&embed_cfg)
        .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))
}

/// Build `VectorStoreProviderConfig` from application config and resolve the provider.
fn resolve_vector_store_from_config(
    config: &mcb_infrastructure::config::AppConfig,
) -> mcb_domain::Result<Arc<dyn VectorStoreProvider>> {
    let mut vec_cfg = VectorStoreProviderConfig::new(
        config
            .providers
            .vector_store
            .provider
            .as_deref()
            .unwrap_or("null"),
    );
    if let Some(ref v) = config.providers.vector_store.address {
        vec_cfg = vec_cfg.with_uri(v.clone());
    }
    if let Some(ref v) = config.providers.vector_store.collection {
        vec_cfg = vec_cfg.with_collection(v.clone());
    }
    if let Some(d) = config.providers.vector_store.dimensions {
        vec_cfg = vec_cfg.with_dimensions(d);
    }
    resolve_vector_store_provider(&vec_cfg)
        .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))
}
