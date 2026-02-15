//! Moka in-memory cache provider
//!
//! High-performance, concurrent in-memory cache implementation using Moka.
//!
//! ## Features
//!
//! - High-performance concurrent cache
//! - Configurable capacity and TTL
//! - Automatic eviction of expired entries
//!
//! ## Example
//!
//! ```no_run
//! use mcb_providers::cache::MokaCacheProvider;
//! use std::time::Duration;
//!
//! let provider = MokaCacheProvider::with_config(1000, Duration::from_secs(300));
//! ```

use std::time::{Duration, Instant};

use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::cache::{CacheEntryConfig, CacheProvider, CacheStats};
use moka::future::Cache;

/// Moka-based in-memory cache provider
///
/// Uses the Moka crate for high-performance concurrent caching.
/// Supports configurable capacity and TTL.
///
/// Created at runtime via factory pattern.
#[derive(Clone)]
pub struct MokaCacheProvider {
    cache: Cache<String, CachedValue>,
    max_size: usize,
}

#[derive(Clone)]
struct CachedValue {
    bytes: Vec<u8>,
    expires_at: Option<Instant>,
}

impl MokaCacheProvider {
    /// Creates a provider with the configured cache capacity.
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self::with_capacity(max_size)
    }

    /// Create a new Moka cache provider with specified capacity
    #[must_use]
    pub fn with_capacity(max_size: usize) -> Self {
        let cache = Cache::builder().max_capacity(max_size as u64).build();

        Self { cache, max_size }
    }

    /// Create a new Moka cache provider with custom configuration
    #[must_use]
    pub fn with_config(max_size: usize, time_to_live: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_size as u64)
            .time_to_live(time_to_live)
            .build();

        Self { cache, max_size }
    }

    /// Get the maximum capacity of the cache
    #[must_use]
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    async fn set(&self, key: &str, bytes: Vec<u8>, config: CacheEntryConfig) -> Result<()> {
        // Check if the value exceeds our size limit
        if bytes.len() > self.max_size {
            return Err(Error::Infrastructure {
                message: format!(
                    "Cache value size {} exceeds maximum size {}",
                    bytes.len(),
                    self.max_size
                ),
                source: None,
            });
        }

        let expires_at = config.ttl.and_then(|ttl| Instant::now().checked_add(ttl));

        self.cache
            .insert(key.to_owned(), CachedValue { bytes, expires_at })
            .await;
        Ok(())
    }
}

#[async_trait]
impl CacheProvider for MokaCacheProvider {
    async fn get_json(&self, key: &str) -> Result<Option<String>> {
        if let Some(cached_value) = self.cache.get(key).await {
            if cached_value
                .expires_at
                .is_some_and(|expires_at| Instant::now() >= expires_at)
            {
                self.cache.invalidate(key).await;
                return Ok(None);
            }

            let json =
                String::from_utf8(cached_value.bytes).map_err(|e| Error::Infrastructure {
                    message: format!("Invalid UTF-8 in cached value: {e}"),
                    source: Some(Box::new(e)),
                })?;
            Ok(Some(json))
        } else {
            Ok(None)
        }
    }

    async fn set_json(&self, key: &str, value: &str, config: CacheEntryConfig) -> Result<()> {
        self.set(key, value.as_bytes().to_vec(), config).await
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let existed = self.cache.contains_key(key);
        self.cache.invalidate(key).await;
        Ok(existed)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.cache.contains_key(key))
    }

    async fn clear(&self) -> Result<()> {
        self.cache.invalidate_all();
        self.cache.run_pending_tasks().await;
        Ok(())
    }

    async fn stats(&self) -> Result<CacheStats> {
        // Run pending tasks to ensure entry_count is accurate
        self.cache.run_pending_tasks().await;
        let entries = self.cache.entry_count();

        Ok(CacheStats {
            hits: 0,   // Moka doesn't track hits/misses
            misses: 0, // Moka doesn't track hits/misses
            entries,
            hit_rate: 0.0, // Unknown
            bytes_used: 0, // Unknown
        })
    }

    async fn size(&self) -> Result<usize> {
        // Run pending tasks to ensure entry_count is accurate
        self.cache.run_pending_tasks().await;
        Ok(self.cache.entry_count() as usize)
    }

    fn provider_name(&self) -> &str {
        "moka"
    }
}

impl std::fmt::Debug for MokaCacheProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MokaCacheProvider")
            .field("max_size", &self.max_size)
            .field("entries", &self.cache.entry_count())
            .finish()
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_domain::registry::cache::{CACHE_PROVIDERS, CacheProviderConfig, CacheProviderEntry};

/// Factory function for creating Moka cache provider instances.
fn moka_cache_factory(
    config: &CacheProviderConfig,
) -> std::result::Result<Arc<dyn CacheProvider>, String> {
    let max_size = config
        .max_size
        .ok_or_else(|| "Moka cache provider requires max_size in config".to_owned())?;
    let provider = MokaCacheProvider::new(max_size);
    Ok(Arc::new(provider))
}

#[linkme::distributed_slice(CACHE_PROVIDERS)]
static MOKA_PROVIDER: CacheProviderEntry = CacheProviderEntry {
    name: "moka",
    description: "Moka high-performance in-memory cache",
    factory: moka_cache_factory,
};
