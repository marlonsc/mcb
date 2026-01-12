//! Advanced distributed caching system with multiple backends
//!
//! Provides pluggable cache providers supporting:
//! - Moka: Local in-memory cache (single-node, default)
//! - Redis: Distributed cache (cluster deployments)
//!
//! ## Architecture
//!
//! Application code should depend on the `CacheProvider` trait, not concrete implementations:
//!
//! ```ignore
//! use crate::infrastructure::cache::SharedCacheProvider;
//!
//! async fn my_function(cache: SharedCacheProvider) {
//!     // Works with any cache backend
//!     cache.set("ns", "key", vec![1,2,3], Duration::from_secs(60)).await?;
//! }
//! ```

mod config;
mod factory;
mod provider;
mod providers;
mod queue;

// Re-export configuration types
pub use config::{
    CacheBackendConfig, CacheConfig, CacheEntry, CacheNamespaceConfig, CacheNamespacesConfig,
    CacheResult,
};

// Re-export provider trait and implementations
pub use provider::{
    CacheProvider, CacheStats, HealthStatus, NullCacheProvider,
    SharedCacheProvider,
};
pub use providers::moka::MokaCacheProvider;
pub use providers::redis::RedisCacheProvider;

// Re-export queue extension trait
pub use queue::CacheProviderQueue;

// Re-export factory function (only place where concrete implementations are created)
pub use factory::create_cache_provider;

use crate::domain::error::Error;

/// Convert Redis errors to domain errors in the infrastructure layer
impl From<::redis::RedisError> for Error {
    fn from(err: ::redis::RedisError) -> Self {
        Self::Cache {
            message: err.to_string(),
        }
    }
}
