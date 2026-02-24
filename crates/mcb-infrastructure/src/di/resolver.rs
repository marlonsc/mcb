//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#dependency-injection)
//!
//! Dynamic Provider Resolver
//!
//! Resolves providers by name using the linkme distributed slice registry.
//! No direct knowledge of concrete provider implementations.
//!
//! ## Architecture
//!
//! This module provides the bridge between configuration and provider instances:
//!
//! ```text
//! Config: "embedding.provider = ollama"
//!                    │
//!                    ▼
//! ┌─────────────────────────────────────┐
//! │     resolve_providers(&config)       │
//! └─────────────────────────────────────┘
//!                    │
//!                    ▼
//! ┌─────────────────────────────────────┐
//! │   PROVIDERS.iter()                   │  ← Discovers auto-registered providers
//! └─────────────────────────────────────┘
//!                    │
//!                    ▼
//! ┌─────────────────────────────────────┐
//! │   ResolvedProviders {                │
//! │     embedding: Arc<dyn ...>,         │
//! │     vector_store: Arc<dyn ...>,      │
//! │     cache: Arc<dyn ...>,             │
//! │     language: Arc<dyn ...>,          │
//! │   }                                  │
//! └─────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! use mcb_infrastructure::config::ConfigLoader;
//! use mcb_infrastructure::di::resolve_providers;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ConfigLoader::new().load()?;
//! let providers = resolve_providers(&config)?;
//!
//! // Use providers for embedding operations
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    CacheProvider as CacheProviderTrait, EmbeddingProvider, LanguageChunkingProvider,
    VectorStoreProvider,
};
use mcb_domain::registry::cache::{CacheProviderConfig, resolve_cache_provider};
use mcb_domain::registry::embedding::{EmbeddingProviderConfig, resolve_embedding_provider};
use mcb_domain::registry::language::{LanguageProviderConfig, resolve_language_provider};
use mcb_domain::registry::vector_store::{
    VectorStoreProviderConfig, resolve_vector_store_provider,
};

use crate::config::AppConfig;
use crate::di::provider_resolvers::{
    embedding_config_to_registry, vector_store_config_to_registry,
};

/// Resolved providers from configuration
///
/// Contains all provider instances resolved from application configuration.
/// These providers are ready to use and have been fully initialized.
#[derive(Clone)]
pub struct ResolvedProviders {
    /// Embedding provider for text-to-vector conversion
    pub embedding: Arc<dyn EmbeddingProvider>,
    /// Vector store for similarity search
    pub vector_store: Arc<dyn VectorStoreProvider>,
    /// Cache provider for performance optimization
    pub cache: Arc<dyn CacheProviderTrait>,
    /// Language chunking provider for code parsing
    pub language: Arc<dyn LanguageChunkingProvider>,
}

impl std::fmt::Debug for ResolvedProviders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedProviders")
            .field("embedding", &self.embedding.provider_name())
            .field("vector_store", &self.vector_store.provider_name())
            .field("cache", &self.cache.provider_name())
            .field("language", &self.language.provider_name())
            .finish()
    }
}

/// Resolve all providers from application configuration
///
/// Queries the linkme registry to find and instantiate providers
/// based on the names specified in configuration.
///
/// # Arguments
/// * `config` - Application configuration containing provider names
///
/// # Errors
///
/// Returns an error if any provider cannot be found or instantiated from config.
pub fn resolve_providers(config: &AppConfig) -> Result<ResolvedProviders> {
    // Get embedding config: prefer direct config (env vars), fallback to named config
    let embedding_config = if config.providers.embedding.provider.is_some() {
        // Direct config (e.g. settings.providers.embedding.provider in YAML)
        EmbeddingProviderConfig {
            provider: config
                .providers
                .embedding
                .provider
                .clone()
                .unwrap_or_default(),
            model: config.providers.embedding.model.clone(),
            api_key: config.providers.embedding.api_key.clone(),
            base_url: config.providers.embedding.base_url.clone(),
            dimensions: config.providers.embedding.dimensions,
            cache_dir: config.providers.embedding.cache_dir.clone(),
            extra: Default::default(),
        }
    } else {
        // Fallback to named config (YAML settings)
        config
            .providers
            .embedding
            .configs
            .values()
            .next()
            .map(embedding_config_to_registry)
            .ok_or_else(|| {
                Error::configuration(
                    "No embedding provider configured; set [providers.embedding.configs.default] in config",
                )
            })?
    };

    // Get vector store config: prefer direct config (env vars), fallback to named config
    let vector_store_config = if config.providers.vector_store.provider.is_some() {
        // Direct config (e.g. settings.providers.vector_store.provider in YAML)
        VectorStoreProviderConfig {
            provider: config
                .providers
                .vector_store
                .provider
                .clone()
                .unwrap_or_default(),
            uri: config.providers.vector_store.address.clone(),
            collection: config.providers.vector_store.collection.clone(),
            dimensions: config.providers.vector_store.dimensions,
            api_key: None,
            encrypted: None,
            encryption_key: None,
            extra: Default::default(),
        }
    } else {
        // Fallback to named config (YAML settings)
        config
            .providers
            .vector_store
            .configs
            .values()
            .next()
            .map(vector_store_config_to_registry)
            .ok_or_else(|| {
                Error::configuration(
                    "No vector store provider configured; set [providers.vector_store.configs.default] in config",
                )
            })?
    };

    // Cache config from system.infrastructure.cache
    // Use as_str() to decouple from concrete enum - enables registry-based resolution
    let cache_provider_name = config.system.infrastructure.cache.provider.as_str();

    let cache_config = CacheProviderConfig {
        provider: cache_provider_name.to_owned(),
        uri: config.system.infrastructure.cache.redis_url.clone(),
        max_size: Some(config.system.infrastructure.cache.max_size),
        ttl_secs: Some(config.system.infrastructure.cache.default_ttl_secs),
        namespace: Some(config.system.infrastructure.cache.namespace.clone()),
        extra: Default::default(),
    };

    // Language config - use "universal" as default
    let language_config = LanguageProviderConfig::new("universal");

    // Resolve each provider from registry
    let embedding = resolve_embedding_provider(&embedding_config)
        .map_err(|e| Error::configuration(format!("Failed to resolve embedding provider: {e}")))?;

    let vector_store = resolve_vector_store_provider(&vector_store_config).map_err(|e| {
        Error::configuration(format!("Failed to resolve vector store provider: {e}"))
    })?;

    let cache = resolve_cache_provider(&cache_config)
        .map_err(|e| Error::configuration(format!("Failed to resolve cache provider: {e}")))?;

    let language = resolve_language_provider(&language_config)
        .map_err(|e| Error::configuration(format!("Failed to resolve language provider: {e}")))?;

    Ok(ResolvedProviders {
        embedding,
        vector_store,
        cache,
        language,
    })
}

/// List all available providers across all categories
///
/// Useful for CLI help, admin UI, and configuration validation.
///
/// # Returns
/// Struct containing lists of available providers by category
#[must_use]
pub fn list_available_providers() -> AvailableProviders {
    AvailableProviders {
        embedding: mcb_domain::registry::embedding::list_embedding_providers(),
        vector_store: mcb_domain::registry::vector_store::list_vector_store_providers(),
        cache: mcb_domain::registry::cache::list_cache_providers(),
        language: mcb_domain::registry::language::list_language_providers(),
    }
}

/// Available providers by category
#[derive(Debug, Clone)]
pub struct AvailableProviders {
    /// Available embedding providers (name, description)
    pub embedding: Vec<(&'static str, &'static str)>,
    /// Available vector store providers (name, description)
    pub vector_store: Vec<(&'static str, &'static str)>,
    /// Available cache providers (name, description)
    pub cache: Vec<(&'static str, &'static str)>,
    /// Available language chunking providers (name, description)
    pub language: Vec<(&'static str, &'static str)>,
}

impl AvailableProviders {
    fn write_section(
        f: &mut std::fmt::Formatter<'_>,
        title: &str,
        providers: &[(&'static str, &'static str)],
    ) -> std::fmt::Result {
        writeln!(f, "{title}:")?;
        for (name, desc) in providers {
            writeln!(f, "  - {name}: {desc}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for AvailableProviders {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Available Providers:")?;
        writeln!(f)?;

        Self::write_section(f, "Embedding Providers", &self.embedding)?;
        writeln!(f)?;

        Self::write_section(f, "Vector Store Providers", &self.vector_store)?;
        writeln!(f)?;

        Self::write_section(f, "Cache Providers", &self.cache)?;
        writeln!(f)?;

        Self::write_section(f, "Language Providers", &self.language)?;

        Ok(())
    }
}
