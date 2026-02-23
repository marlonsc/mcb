//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md#dependency-injection)
//!
//! Provider Resolvers - Components for resolving providers from linkme registry
//!
//! These components wrap the linkme registry resolution and can be injected
//! into other services.
//!
//! ## Pattern
//!
//! ```text
//! AppConfig (injected) → Resolver → linkme registry → Arc<dyn Provider>
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use mcb_domain::ports::{
    CacheProvider, EmbeddingProvider, LanguageChunkingProvider, VectorStoreProvider,
};
use mcb_domain::registry::cache::{CacheProviderConfig, resolve_cache_provider};
use mcb_domain::registry::embedding::{EmbeddingProviderConfig, resolve_embedding_provider};
use mcb_domain::registry::language::{LanguageProviderConfig, resolve_language_provider};
use mcb_domain::registry::vector_store::{
    VectorStoreProviderConfig, resolve_vector_store_provider,
};
use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};

use crate::config::AppConfig;
use crate::constants::providers::{
    DEFAULT_DB_CONFIG_NAME, FALLBACK_EMBEDDING_PROVIDER, FALLBACK_VECTOR_STORE_PROVIDER,
};

macro_rules! impl_resolver_common {
    ($resolver:ident) => {
        impl $resolver {
            #[must_use]
            /// Creates a new resolver with the provided application config.
            pub fn new(config: Arc<AppConfig>) -> Self {
                Self { config }
            }
        }

        impl std::fmt::Debug for $resolver {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($resolver)).finish()
            }
        }
    };
}

// ============================================================================
// Embedding Provider Resolver
// ============================================================================

/// Resolver component for embedding providers
///
/// Uses the linkme registry to resolve embedding providers by name.
/// Can resolve from current config or from an override config.
pub struct EmbeddingProviderResolver {
    config: Arc<AppConfig>,
}

impl EmbeddingProviderResolver {
    /// Resolve provider from current application config
    ///
    /// # Errors
    ///
    /// Returns an error if no embedding provider is configured or resolution fails.
    pub fn resolve_from_config(&self) -> mcb_domain::error::Result<Arc<dyn EmbeddingProvider>> {
        // First, check direct config (e.g. settings.providers.embedding.provider in YAML)
        if let Some(ref provider_name) = self.config.providers.embedding.provider {
            let mut registry_config = EmbeddingProviderConfig::new(provider_name);
            if let Some(ref model) = self.config.providers.embedding.model {
                registry_config.model = Some(model.clone());
            }
            if let Some(ref base_url) = self.config.providers.embedding.base_url {
                registry_config.base_url = Some(base_url.clone());
            }
            if let Some(ref api_key) = self.config.providers.embedding.api_key {
                registry_config.api_key = Some(api_key.clone());
            }
            if let Some(dimensions) = self.config.providers.embedding.dimensions {
                registry_config.dimensions = Some(dimensions);
            }
            if let Some(ref cache_dir) = self.config.providers.embedding.cache_dir {
                registry_config.cache_dir = Some(cache_dir.clone());
            }
            return resolve_embedding_provider(&registry_config);
        }

        resolve_named_config_or_fallback(
            &self.config.providers.embedding.configs,
            embedding_config_to_registry,
            resolve_embedding_provider,
            || {
                resolve_embedding_provider(&EmbeddingProviderConfig::new(
                    FALLBACK_EMBEDDING_PROVIDER,
                ))
            },
        )
    }

    /// Resolve provider from override config (for admin API)
    ///
    /// # Errors
    ///
    /// Returns an error if the embedding provider cannot be resolved.
    pub fn resolve_from_override(
        &self,
        override_config: &EmbeddingProviderConfig,
    ) -> mcb_domain::error::Result<Arc<dyn EmbeddingProvider>> {
        resolve_embedding_provider(override_config)
    }

    /// List available embedding providers
    #[must_use]
    pub fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        mcb_domain::registry::embedding::list_embedding_providers()
    }
}
impl_resolver_common!(EmbeddingProviderResolver);

// ============================================================================
// Vector Store Provider Resolver
// ============================================================================

/// Resolver component for vector store providers
///
/// Uses the linkme registry to resolve vector store providers by name.
/// Can resolve from current config or from an override config.
pub struct VectorStoreProviderResolver {
    config: Arc<AppConfig>,
}

impl VectorStoreProviderResolver {
    /// Resolve provider from current application config
    ///
    /// # Errors
    ///
    /// Returns an error if no matching vector store provider is found in the registry.
    pub fn resolve_from_config(&self) -> mcb_domain::error::Result<Arc<dyn VectorStoreProvider>> {
        // First, check direct config (e.g. settings.providers.vector_store.provider in YAML)
        if let Some(ref provider_name) = self.config.providers.vector_store.provider {
            let mut registry_config = VectorStoreProviderConfig::new(provider_name);
            if let Some(ref address) = self.config.providers.vector_store.address {
                registry_config.uri = Some(address.clone());
            }
            if let Some(dimensions) = self.config.providers.vector_store.dimensions {
                registry_config.dimensions = Some(dimensions);
            }
            if let Some(ref collection) = self.config.providers.vector_store.collection {
                registry_config.collection = Some(collection.clone());
            }
            return resolve_vector_store_provider(&registry_config);
        }

        resolve_named_config_or_fallback(
            &self.config.providers.vector_store.configs,
            vector_store_config_to_registry,
            resolve_vector_store_provider,
            || {
                resolve_vector_store_provider(&VectorStoreProviderConfig::new(
                    FALLBACK_VECTOR_STORE_PROVIDER,
                ))
            },
        )
    }

    /// Resolve provider from override config (for admin API)
    ///
    /// # Errors
    ///
    /// Returns an error if the vector store provider cannot be resolved.
    pub fn resolve_from_override(
        &self,
        override_config: &VectorStoreProviderConfig,
    ) -> mcb_domain::error::Result<Arc<dyn VectorStoreProvider>> {
        resolve_vector_store_provider(override_config)
    }

    /// List available vector store providers
    #[must_use]
    pub fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        mcb_domain::registry::vector_store::list_vector_store_providers()
    }
}

trait ProviderNamed {
    fn provider_name(&self) -> &str;
}

impl ProviderNamed for EmbeddingConfig {
    fn provider_name(&self) -> &str {
        &self.provider
    }
}

impl ProviderNamed for VectorStoreConfig {
    fn provider_name(&self) -> &str {
        &self.provider
    }
}

fn resolve_named_config_or_fallback<TConfig, TRegistry, TProvider, TBuild, TResolve, TFallback>(
    configs: &HashMap<String, TConfig>,
    to_registry: TBuild,
    resolve: TResolve,
    fallback: TFallback,
) -> mcb_domain::error::Result<TProvider>
where
    TConfig: ProviderNamed,
    TBuild: Fn(&TConfig) -> TRegistry,
    TResolve: Fn(&TRegistry) -> mcb_domain::error::Result<TProvider>,
    TFallback: FnOnce() -> mcb_domain::error::Result<TProvider>,
{
    if let Some(default_config) = configs.get(DEFAULT_DB_CONFIG_NAME) {
        let selected_config = configs
            .get(default_config.provider_name())
            .unwrap_or(default_config);
        let registry_config = to_registry(selected_config);
        resolve(&registry_config)
    } else {
        fallback()
    }
}

impl_resolver_common!(VectorStoreProviderResolver);

// ============================================================================
// Cache Provider Resolver
// ============================================================================

/// Resolver component for cache providers
///
/// Uses the linkme registry to resolve cache providers by name.
/// Can resolve from current config or from an override config.
pub struct CacheProviderResolver {
    config: Arc<AppConfig>,
}

impl CacheProviderResolver {
    /// Resolve provider from current application config
    ///
    /// # Errors
    ///
    /// Returns an error if no matching cache provider is found in the registry.
    pub fn resolve_from_config(&self) -> mcb_domain::error::Result<Arc<dyn CacheProvider>> {
        let cache_provider_name = match &self.config.system.infrastructure.cache.provider {
            crate::config::CacheProvider::Moka => "moka",
            crate::config::CacheProvider::Redis => "redis",
        };

        let registry_config = CacheProviderConfig {
            provider: cache_provider_name.to_owned(),
            uri: self.config.system.infrastructure.cache.redis_url.clone(),
            max_size: Some(self.config.system.infrastructure.cache.max_size),
            ttl_secs: Some(self.config.system.infrastructure.cache.default_ttl_secs),
            namespace: Some(self.config.system.infrastructure.cache.namespace.clone()),
            extra: Default::default(),
        };

        resolve_cache_provider(&registry_config)
    }

    /// Resolve provider from override config (for admin API)
    ///
    /// # Errors
    ///
    /// Returns an error if the cache provider cannot be resolved.
    pub fn resolve_from_override(
        &self,
        override_config: &CacheProviderConfig,
    ) -> mcb_domain::error::Result<Arc<dyn CacheProvider>> {
        resolve_cache_provider(override_config)
    }

    /// List available cache providers
    #[must_use]
    pub fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        mcb_domain::registry::cache::list_cache_providers()
    }
}
impl_resolver_common!(CacheProviderResolver);

// ============================================================================
// Language Provider Resolver
// ============================================================================

/// Resolver component for language chunking providers
///
/// Uses the linkme registry to resolve language providers by name.
/// Can resolve from current config or from an override config.
pub struct LanguageProviderResolver {
    config: Arc<AppConfig>,
}

impl LanguageProviderResolver {
    /// Resolve provider from current application config
    ///
    /// # Errors
    ///
    /// Returns an error if the language provider cannot be resolved.
    pub fn resolve_from_config(
        &self,
    ) -> mcb_domain::error::Result<Arc<dyn LanguageChunkingProvider>> {
        // Use config so the field is not dead; language provider is "universal" for now
        let _ = self.config.as_ref();
        let registry_config = LanguageProviderConfig::new("universal");
        resolve_language_provider(&registry_config)
    }

    /// Resolve provider from override config (for admin API)
    ///
    /// # Errors
    ///
    /// Returns an error if the language provider cannot be resolved.
    pub fn resolve_from_override(
        &self,
        override_config: &LanguageProviderConfig,
    ) -> mcb_domain::error::Result<Arc<dyn LanguageChunkingProvider>> {
        resolve_language_provider(override_config)
    }

    /// List available language providers
    #[must_use]
    pub fn list_available(&self) -> Vec<(&'static str, &'static str)> {
        mcb_domain::registry::language::list_language_providers()
    }
}
impl_resolver_common!(LanguageProviderResolver);

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert domain `EmbeddingConfig` to registry `EmbeddingProviderConfig`
pub(crate) fn embedding_config_to_registry(config: &EmbeddingConfig) -> EmbeddingProviderConfig {
    EmbeddingProviderConfig {
        provider: config.provider.clone(),
        model: Some(config.model.clone()),
        api_key: config.api_key.clone(),
        base_url: config.base_url.clone(),
        dimensions: config.dimensions,
        cache_dir: None,
        extra: Default::default(),
    }
}

/// Convert domain `VectorStoreConfig` to registry `VectorStoreProviderConfig`
pub(crate) fn vector_store_config_to_registry(
    config: &VectorStoreConfig,
) -> VectorStoreProviderConfig {
    VectorStoreProviderConfig {
        provider: config.provider.clone(),
        uri: config.address.clone(),
        collection: config.collection.clone(),
        dimensions: config.dimensions,
        api_key: config.token.clone(),
        encrypted: None,
        encryption_key: None,
        extra: Default::default(),
    }
}
