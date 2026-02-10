//! Provider Admin Services - Runtime provider switching via API
//!
//! These components allow switching providers at runtime without restarting
//! the application. Used by admin API endpoints.
//!
//! ## Pattern
//!
//! ```text
//! Admin API → AdminService.switch_provider() → Resolver → Handle.set()
//! ```

use std::sync::Arc;

pub use mcb_domain::ports::admin::{
    CacheAdminInterface, DependencyHealth, DependencyHealthCheck, EmbeddingAdminInterface,
    LanguageAdminInterface, LifecycleManaged, PortServiceState, ProviderInfo,
    VectorStoreAdminInterface,
};
use mcb_domain::ports::providers::{
    CacheProvider, EmbeddingProvider, LanguageChunkingProvider, VectorStoreProvider,
};
use mcb_domain::registry::cache::CacheProviderConfig;
use mcb_domain::registry::embedding::EmbeddingProviderConfig;
use mcb_domain::registry::language::LanguageProviderConfig;
use mcb_domain::registry::vector_store::VectorStoreProviderConfig;

use super::handle::Handle;
use super::handles::{CacheHandleExt, EmbeddingHandleExt};
use super::provider_resolvers::{
    CacheProviderResolver, EmbeddingProviderResolver, LanguageProviderResolver,
    VectorStoreProviderResolver,
};

// ============================================================================
// Resolver Trait - Common Interface for All Provider Resolvers
// ============================================================================

/// Common interface for provider resolvers
///
/// This trait abstracts the resolution logic so AdminService can work
/// with any resolver type generically.
///
/// # Example
///
/// ```
/// use mcb_infrastructure::di::ProviderResolver;
///
/// fn list_providers<R, P, C>(resolver: &R) -> Vec<String>
/// where
///     R: ProviderResolver<P, C>,
///     P: ?Sized + Send + Sync,
/// {
///     resolver.list_available()
///         .into_iter()
///         .map(|(name, _)| name.to_string())
///         .collect()
/// }
/// ```
pub trait ProviderResolver<P: ?Sized + Send + Sync, C>: Send + Sync {
    /// Resolve provider from current application config
    fn resolve_from_config(&self) -> mcb_domain::error::Result<Arc<P>>;
    /// Resolve provider from override config (for admin API)
    fn resolve_from_override(&self, config: &C) -> mcb_domain::error::Result<Arc<P>>;
    /// List available providers
    fn list_available(&self) -> Vec<(&'static str, &'static str)>;
}

// ============================================================================
// Resolver Trait Implementations
// ============================================================================

macro_rules! impl_provider_resolver {
    ($resolver:ty, $provider:ty, $config:ty) => {
        impl ProviderResolver<$provider, $config> for $resolver {
            fn resolve_from_config(&self) -> mcb_domain::error::Result<Arc<$provider>> {
                <$resolver>::resolve_from_config(self)
            }

            fn resolve_from_override(
                &self,
                config: &$config,
            ) -> mcb_domain::error::Result<Arc<$provider>> {
                <$resolver>::resolve_from_override(self, config)
            }

            fn list_available(&self) -> Vec<(&'static str, &'static str)> {
                <$resolver>::list_available(self)
            }
        }
    };
}

impl_provider_resolver!(
    EmbeddingProviderResolver,
    dyn EmbeddingProvider,
    EmbeddingProviderConfig
);
impl_provider_resolver!(
    VectorStoreProviderResolver,
    dyn VectorStoreProvider,
    VectorStoreProviderConfig
);
impl_provider_resolver!(
    CacheProviderResolver,
    dyn CacheProvider,
    CacheProviderConfig
);
impl_provider_resolver!(
    LanguageProviderResolver,
    dyn LanguageChunkingProvider,
    LanguageProviderConfig
);

// ============================================================================
// Generic Admin Service
// ============================================================================

/// Generic admin service for managing providers at runtime
///
/// This struct provides the core admin functionality for any provider type.
/// Specific admin services are type aliases with the appropriate resolver
/// and provider types.
///
/// # Type Parameters
///
/// * `R` - Resolver type that implements `ProviderResolver<P, C>`
/// * `P` - Provider trait type (e.g., `dyn EmbeddingProvider`)
/// * `C` - Config type for the provider
pub struct AdminService<R, P: ?Sized + Send + Sync, C> {
    name: String,
    resolver: Arc<R>,
    handle: Arc<Handle<P>>,
    _config_marker: std::marker::PhantomData<C>,
}

impl<R, P, C> AdminService<R, P, C>
where
    R: ProviderResolver<P, C>,
    P: ?Sized + Send + Sync,
{
    /// Create a new admin service
    pub fn new(name: impl Into<String>, resolver: Arc<R>, handle: Arc<Handle<P>>) -> Self {
        Self {
            name: name.into(),
            resolver,
            handle,
            _config_marker: std::marker::PhantomData,
        }
    }

    /// List all available providers
    pub fn list_providers(&self) -> Vec<ProviderInfo> {
        self.resolver
            .list_available()
            .into_iter()
            .map(|(name, description)| ProviderInfo {
                name: name.to_string(),
                description: description.to_string(),
            })
            .collect()
    }

    /// Switch to a different provider
    ///
    /// # Arguments
    /// * `config` - Configuration for the new provider
    ///
    /// # Returns
    /// * `Ok(())` - Provider switched successfully
    /// * `Err(String)` - Failed to switch (provider not found, config invalid, etc.)
    pub fn switch_provider(&self, config: &C) -> Result<(), String> {
        let new_provider = self
            .resolver
            .resolve_from_override(config)
            .map_err(|e| e.to_string())?;
        self.handle.set(new_provider);
        Ok(())
    }

    /// Reload provider from current application config
    pub fn reload_from_config(&self) -> Result<(), String> {
        let provider = self
            .resolver
            .resolve_from_config()
            .map_err(|e| e.to_string())?;
        self.handle.set(provider);
        Ok(())
    }
}

#[async_trait::async_trait]
impl<R, P, C> LifecycleManaged for AdminService<R, P, C>
where
    R: ProviderResolver<P, C>,
    P: ?Sized + Send + Sync,
    C: Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn start(&self) -> mcb_domain::error::Result<()> {
        // Admin services are always "started" as they manage handles
        Ok(())
    }

    async fn stop(&self) -> mcb_domain::error::Result<()> {
        // Cannot stop an admin service
        Ok(())
    }

    fn state(&self) -> PortServiceState {
        // Always considered running as they are infrastructure
        PortServiceState::Running
    }

    async fn health_check(&self) -> DependencyHealthCheck {
        DependencyHealthCheck {
            name: self.name.clone(),
            status: DependencyHealth::Healthy,
            message: Some(format!("Service active: {}", self.name)),
            latency_ms: None,
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

impl<R, P, C> std::fmt::Debug for AdminService<R, P, C>
where
    P: ?Sized + Send + Sync,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdminService").finish()
    }
}

// ============================================================================
// Type Aliases (Backward Compatibility)
// ============================================================================

/// Admin service for managing embedding providers at runtime
pub type EmbeddingAdminService =
    AdminService<EmbeddingProviderResolver, dyn EmbeddingProvider, EmbeddingProviderConfig>;

/// Admin service for managing vector store providers at runtime
pub type VectorStoreAdminService =
    AdminService<VectorStoreProviderResolver, dyn VectorStoreProvider, VectorStoreProviderConfig>;

/// Admin service for managing cache providers at runtime
pub type CacheAdminService =
    AdminService<CacheProviderResolver, dyn CacheProvider, CacheProviderConfig>;

/// Admin service for managing language chunking providers at runtime
pub type LanguageAdminService =
    AdminService<LanguageProviderResolver, dyn LanguageChunkingProvider, LanguageProviderConfig>;

// ============================================================================
// Trait Implementations for Specific Admin Services
// ============================================================================

macro_rules! impl_admin_interface {
    ($service:ty, $trait:ty, $config:ty) => {
        impl $trait for $service {
            fn list_providers(&self) -> Vec<ProviderInfo> {
                AdminService::list_providers(self)
            }

            fn switch_provider(&self, config: $config) -> Result<(), String> {
                AdminService::switch_provider(self, &config)
            }

            fn reload_from_config(&self) -> Result<(), String> {
                AdminService::reload_from_config(self)
            }
        }
    };
    ($service:ty, $trait:ty, $config:ty, with_current_provider) => {
        impl $trait for $service {
            fn list_providers(&self) -> Vec<ProviderInfo> {
                AdminService::list_providers(self)
            }

            fn current_provider(&self) -> String {
                self.handle.provider_name()
            }

            fn switch_provider(&self, config: $config) -> Result<(), String> {
                AdminService::switch_provider(self, &config)
            }

            fn reload_from_config(&self) -> Result<(), String> {
                AdminService::reload_from_config(self)
            }
        }
    };
}

impl_admin_interface!(
    EmbeddingAdminService,
    EmbeddingAdminInterface,
    EmbeddingProviderConfig,
    with_current_provider
);
impl_admin_interface!(
    VectorStoreAdminService,
    VectorStoreAdminInterface,
    VectorStoreProviderConfig
);
impl_admin_interface!(
    CacheAdminService,
    CacheAdminInterface,
    CacheProviderConfig,
    with_current_provider
);
impl_admin_interface!(
    LanguageAdminService,
    LanguageAdminInterface,
    LanguageProviderConfig
);
