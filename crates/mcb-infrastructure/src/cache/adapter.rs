//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../../../docs/modules/infrastructure.md)
//!
//! Cache Adapter
//!
//! Bridges the framework cache to MCB's `CacheProvider` trait.

use std::sync::Arc;

use async_trait::async_trait;
use loco_rs::cache::Cache;
use mcb_domain::error::Result;
use mcb_domain::ports::{CacheEntryConfig, CacheProvider, CacheStats};

/// Adapts the framework cache to MCB's `CacheProvider` port.
#[derive(Clone)]
pub struct CacheAdapter {
    cache: Arc<Cache>,
}

impl std::fmt::Debug for CacheAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CacheAdapter")
    }
}

impl CacheAdapter {
    /// Wrap a framework cache instance.
    #[must_use]
    pub fn new(cache: Arc<Cache>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl CacheProvider for CacheAdapter {
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
        if let Some(ttl) = config.ttl {
            self.cache
                .insert_with_expiry(key, &value.to_owned(), ttl)
                .await
        } else {
            self.cache.insert(key, &value.to_owned()).await
        }
        .map_err(|e| mcb_domain::error::Error::Infrastructure {
            message: format!("Cache set failed: {e}"),
            source: Some(Box::new(e)),
        })
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let existed = self.cache.contains_key(key).await.map_err(|e| {
            mcb_domain::error::Error::Infrastructure {
                message: format!("Cache exists check before delete failed: {e}"),
                source: Some(Box::new(e)),
            }
        })?;
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
        Err(mcb_domain::error::Error::Infrastructure {
            message: "Cache statistics not available: underlying provider does not expose metrics"
                .to_owned(),
            source: None,
        })
    }

    async fn size(&self) -> Result<usize> {
        Err(mcb_domain::error::Error::Infrastructure {
            message: "Cache size not available: underlying provider does not expose metrics"
                .to_owned(),
            source: None,
        })
    }

    fn provider_name(&self) -> &str {
        "cache-adapter"
    }
}

impl From<Arc<Cache>> for CacheAdapter {
    fn from(cache: Arc<Cache>) -> Self {
        Self::new(cache)
    }
}
