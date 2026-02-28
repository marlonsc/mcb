//! Service registry and linkme registration for `MemoryService`.
//!
//! Handles dependency resolution and service builder registration.

use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::MemoryServiceInterface;
use mcb_domain::registry::services::{
    MEMORY_SERVICE_NAME, SERVICES_REGISTRY, ServiceBuilder, ServiceRegistryEntry,
};

use super::MemoryServiceImpl;

/// Registry provider name for `SeaORM` database repositories.
const DATABASE_PROVIDER: &str = "seaorm";

/// Default namespace for database repositories.
const DEFAULT_NAMESPACE: &str = "default";

/// Default embedding provider name (null provider when not configured).
const DEFAULT_EMBEDDING_PROVIDER: &str = "null";

/// Default vector store provider name (null provider when not configured).
const DEFAULT_VECTOR_STORE_PROVIDER: &str = "null";

/// Build a `MemoryService` from the service resolution context.
fn build_memory_service_from_registry(
    context: &dyn std::any::Any,
) -> Result<Arc<dyn MemoryServiceInterface>> {
    let ctx = context
        .downcast_ref::<crate::resolution_context::ServiceResolutionContext>()
        .ok_or_else(|| {
            mcb_domain::error::Error::internal(
                "Memory service builder requires ServiceResolutionContext",
            )
        })?;

    let config = &ctx.config;

    // Resolve embedding provider (same config propagation as context_service)
    let mut embed_cfg = mcb_domain::registry::embedding::EmbeddingProviderConfig::new(
        config
            .providers
            .embedding
            .provider
            .as_deref()
            .unwrap_or(DEFAULT_EMBEDDING_PROVIDER),
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
    let embedding = mcb_domain::registry::embedding::resolve_embedding_provider(&embed_cfg)
        .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))?;

    // Resolve vector store provider
    let mut vec_cfg = mcb_domain::registry::vector_store::VectorStoreProviderConfig::new(
        config
            .providers
            .vector_store
            .provider
            .as_deref()
            .unwrap_or(DEFAULT_VECTOR_STORE_PROVIDER),
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
    let vector_store = mcb_domain::registry::vector_store::resolve_vector_store_provider(&vec_cfg)
        .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))?;

    // Resolve memory repository from database providers
    let repos = mcb_domain::registry::database::resolve_database_repositories(
        DATABASE_PROVIDER,
        Box::new(ctx.db.clone()),
        DEFAULT_NAMESPACE.to_owned(),
    )?;

    Ok(Arc::new(MemoryServiceImpl::new(
        DEFAULT_NAMESPACE.to_owned(),
        repos.memory,
        embedding,
        vector_store,
    )))
}

/// Linkme distributed slice entry for `MemoryService` registration.
#[linkme::distributed_slice(SERVICES_REGISTRY)]
static MEMORY_SERVICE_REGISTRY_ENTRY: ServiceRegistryEntry = ServiceRegistryEntry {
    name: MEMORY_SERVICE_NAME,
    build: ServiceBuilder::Memory(build_memory_service_from_registry),
};
