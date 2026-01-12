//! Cache provider factory
//!
//! Creates appropriate cache provider instances based on configuration.
//! This is the only place in the codebase where concrete cache implementations
//! are instantiated - application code depends only on CacheProvider trait.

use super::config::CacheBackendConfig;
use super::provider::NullCacheProvider;
use super::providers::moka::MokaCacheProvider;
use super::providers::redis::RedisCacheProvider;
use super::{CacheConfig, SharedCacheProvider};
use crate::domain::error::Result;
use std::sync::Arc;
use std::time::Duration;

/// Create a cache provider based on configuration
///
/// This factory function instantiates the appropriate cache provider
/// (Moka, Redis, or Null) based on the configuration settings.
pub async fn create_cache_provider(config: &CacheConfig) -> Result<SharedCacheProvider> {
    // If caching is disabled, return null provider
    if !config.enabled {
        tracing::info!("[CACHE] Caching disabled, using null provider");
        return Ok(Arc::new(NullCacheProvider) as SharedCacheProvider);
    }

    match &config.backend {
        CacheBackendConfig::Local {
            max_entries: _,
            default_ttl_seconds: _,
        } => {
            tracing::info!("[CACHE] Using Moka local cache provider");
            let provider = MokaCacheProvider::new(config.clone())?;
            Ok(Arc::new(provider) as SharedCacheProvider)
        }

        CacheBackendConfig::Redis {
            url,
            pool_size: _,
            default_ttl_seconds,
        } => {
            tracing::info!("[CACHE] Using Redis distributed cache provider");
            let provider =
                RedisCacheProvider::new(url, Duration::from_secs(*default_ttl_seconds)).await?;
            Ok(Arc::new(provider) as SharedCacheProvider)
        }
    }
}
