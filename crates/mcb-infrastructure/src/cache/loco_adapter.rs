//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../../../docs/modules/infrastructure.md)
//!
//! Loco Cache Adapter
//!
//! Adapts Loco's cache API to MCB's `CacheProvider` trait.
//! Replaces the custom cache provider infrastructure with Loco's built-in cache.

use std::sync::Arc;

use async_trait::async_trait;
use loco_rs::cache::Cache;
use mcb_domain::error::Result;
use mcb_domain::ports::{CacheEntryConfig, CacheProvider, CacheStats};

/// Adapter that implements MCB's `CacheProvider` trait using Loco's cache.
///
/// This adapter allows MCB's domain services to use Loco's cache implementation
/// (InMem or Redis) without code changes to the domain layer.
#[derive(Debug, Clone)]
pub struct LocoCacheAdapter {
    cache: Arc<Cache>,
}

impl LocoCacheAdapter {
    /// Create a new Loco cache adapter from Loco's cache.
    #[must_use]
    pub fn new(cache: Arc<Cache>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl CacheProvider for LocoCacheAdapter {
    async fn get_json(&self, key: &str) -> Result<Option<String>> {
        self.cache
            .get::<String>(key)
            .await
            .map_err(|e| mcb_domain::error::Error::Infrastructure {
                message: format!("Cache get failed: {e}"),
                source: Some(Box::new(e)),
            })
    }

    async fn set_json(&self, key: &str, value: &str, config: CacheEntryConfig) -> Result<()> {
        if let Some(ttl_secs) = config.ttl_secs {
            let duration = std::time::Duration::from_secs(ttl_secs);
            self.cache
                .insert_with_expiry(key, &value.to_string(), duration)
                .await
        } else {
            self.cache.insert(key, &value.to_string()).await
        }
        .map_err(|e| mcb_domain::error::Error::Infrastructure {
            message: format!("Cache set failed: {e}"),
            source: Some(Box::new(e)),
        })
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        // Loco's remove returns (), so we check exists first
        let existed = self.cache.contains_key(key).await.unwrap_or(false);
        self.cache
            .remove(key)
            .await
            .map_err(|e| mcb_domain::error::Error::Infrastructure {
                message: format!("Cache delete failed: {e}"),
                source: Some(Box::new(e)),
            })?;
        Ok(existed)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        self.cache
            .contains_key(key)
            .await
            .map_err(|e| mcb_domain::error::Error::Infrastructure {
                message: format!("Cache exists check failed: {e}"),
                source: Some(Box::new(e)),
            })
    }

    async fn clear(&self) -> Result<()> {
        self.cache
            .clear()
            .await
            .map_err(|e| mcb_domain::error::Error::Infrastructure {
                message: format!("Cache clear failed: {e}"),
                source: Some(Box::new(e)),
            })
    }

    async fn stats(&self) -> Result<CacheStats> {
        // Loco's cache doesn't expose detailed stats, return defaults
        Ok(CacheStats {
            hits: 0,
            misses: 0,
            size: self.cache.size().await.unwrap_or(0),
        })
    }

    async fn size(&self) -> Result<usize> {
        self.cache
            .size()
            .await
            .map_err(|e| mcb_domain::error::Error::Infrastructure {
                message: format!("Cache size failed: {e}"),
                source: Some(Box::new(e)),
            })
    }

    fn provider_name(&self) -> &str {
        "loco"
    }
}

impl From<Arc<Cache>> for LocoCacheAdapter {
    fn from(cache: Arc<Cache>) -> Self {
        Self::new(cache)
    }
}
