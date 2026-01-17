//! Cache provider factory
//!
//! Factory for creating cache provider instances based on configuration.
//! Uses cache provider implementations from mcb-providers crate.

use crate::cache::provider::SharedCacheProvider;
use crate::config::data::*;
use mcb_domain::error::Result;
use mcb_providers::cache::{MokaCacheProvider, NullCacheProvider, RedisCacheProvider};

/// Cache provider factory
#[derive(Clone)]
pub struct CacheProviderFactory;

impl CacheProviderFactory {
    /// Create a cache provider from configuration
    pub async fn create_from_config(config: &CacheConfig) -> Result<SharedCacheProvider> {
        use crate::cache::provider::CacheProviderType;

        if !config.enabled {
            return Ok(SharedCacheProvider::new(CacheProviderType::Null(
                NullCacheProvider::new(),
            )));
        }

        let provider = match config.provider {
            crate::config::data::CacheProvider::Moka => {
                CacheProviderType::Moka(MokaCacheProvider::with_capacity(config.max_size))
            }
            crate::config::data::CacheProvider::Redis => {
                let redis_url = config
                    .redis_url
                    .as_deref()
                    .unwrap_or("redis://localhost:6379");
                CacheProviderType::Redis(RedisCacheProvider::new(redis_url)?)
            }
        };

        let mut shared_provider = SharedCacheProvider::new(provider);
        shared_provider.set_namespace(&config.namespace);

        Ok(shared_provider)
    }

    /// Create a Moka cache provider
    pub fn create_moka(max_size: usize) -> SharedCacheProvider {
        use crate::cache::provider::CacheProviderType;
        SharedCacheProvider::new(CacheProviderType::Moka(MokaCacheProvider::with_capacity(
            max_size,
        )))
    }

    /// Create a Redis cache provider
    pub async fn create_redis(connection_string: &str) -> Result<SharedCacheProvider> {
        use crate::cache::provider::CacheProviderType;
        let provider = RedisCacheProvider::new(connection_string)?;
        Ok(SharedCacheProvider::new(CacheProviderType::Redis(provider)))
    }

    /// Create a null cache provider (for testing/disabling cache)
    pub fn create_null() -> SharedCacheProvider {
        use crate::cache::provider::CacheProviderType;
        SharedCacheProvider::new(CacheProviderType::Null(NullCacheProvider::new()))
    }

    /// Create a cache provider with specific namespace
    pub fn with_namespace(
        mut provider: SharedCacheProvider,
        namespace: &str,
    ) -> SharedCacheProvider {
        provider.set_namespace(namespace);
        provider
    }
}
