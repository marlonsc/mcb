use crate::config::AppConfig;
use mcb_domain::ports::{
    EmbeddingProvider, EventBusProvider, HybridSearchProvider, VectorStoreProvider,
};
use mcb_domain::registry::embedding::{EmbeddingProviderConfig, resolve_embedding_provider};
use mcb_domain::registry::vector_store::{
    VectorStoreProviderConfig, resolve_vector_store_provider,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// Context passed to service factory functions during DI resolution.
pub struct ServiceResolutionContext {
    /// Active database connection.
    pub db: DatabaseConnection,
    /// Shared application configuration.
    pub config: Arc<AppConfig>,
    /// Event bus for cross-service communication.
    pub event_bus: Arc<dyn EventBusProvider>,
    /// Shared embedding provider resolved once at startup.
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
    /// Shared vector store provider resolved once at startup.
    pub vector_store_provider: Arc<dyn VectorStoreProvider>,
}

// ---------------------------------------------------------------------------
// Centralized Provider Resolution Helpers
// ---------------------------------------------------------------------------

/// Build [`EmbeddingProviderConfig`] from application config and resolve the provider.
///
/// Centralizes config-to-provider propagation previously duplicated across
/// `composition.rs`, `context_service.rs`, and `memory_service/registry.rs`.
///
/// # Errors
///
/// Returns an error if the embedding provider cannot be resolved from the registry.
pub fn resolve_embedding_from_config(
    config: &AppConfig,
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

/// Build [`VectorStoreProviderConfig`] from application config and resolve the provider.
///
/// Centralizes config-to-provider propagation previously duplicated across
/// `composition.rs`, `context_service.rs`, and `memory_service/registry.rs`.
///
/// # Errors
///
/// Returns an error if the vector store provider cannot be resolved from the registry.
pub fn resolve_vector_store_from_config(
    config: &AppConfig,
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

/// Create a default [`HybridSearchProvider`] with standard BM25/semantic weights.
///
/// This is a lightweight in-memory provider that requires no external dependencies.
/// Used by the composition root to wire hybrid search into the MCP server.
#[must_use]
pub fn create_default_hybrid_search_provider() -> Arc<dyn HybridSearchProvider> {
    Arc::new(mcb_providers::hybrid_search::engine::HybridSearchEngine::new())
}
