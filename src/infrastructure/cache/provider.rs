//! Cache provider trait - abstraction over cache implementations
//!
//! Provides a pluggable interface for cache backends:
//! - Moka: Local in-memory cache (default, single-node)
//! - Redis: Distributed cache (cluster deployments)
//! - Null: No-op cache (testing, when caching disabled)
//!
//! ## Architecture
//!
//! Application code should depend ONLY on `CacheProvider` trait,
//! never on concrete implementations (Moka, Redis). This enables:
//! - Easy provider switching via configuration
//! - Testing with null provider
//! - No coupling to infrastructure libraries

use crate::domain::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of entries
    pub total_entries: usize,
    /// Total cache size in bytes
    pub total_size_bytes: usize,
    /// Cache hit count
    pub hits: u64,
    /// Cache miss count
    pub misses: u64,
    /// Cache hit ratio (0.0 to 1.0)
    pub hit_ratio: f64,
    /// Number of evictions
    pub evictions: u64,
    /// Average access time in microseconds
    pub avg_access_time_us: f64,
}

/// Health check result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Cache is healthy
    Healthy,
    /// Cache is operational but degraded
    Degraded,
    /// Cache is unavailable
    Unhealthy,
}

/// Abstract trait for cache provider implementations
///
/// Enables pluggable cache backends (Moka, Redis, etc.) without
/// coupling application code to infrastructure libraries.
///
/// ## Implementing a Provider
///
/// ```ignore
/// pub struct MyCacheProvider {
///     // Implementation details
/// }
///
/// #[async_trait]
/// impl CacheProvider for MyCacheProvider {
///     async fn get(&self, namespace: &str, key: &str) -> Result<Option<Vec<u8>>> {
///         // Implementation
///     }
///     // ... implement other methods
/// }
/// ```
#[async_trait]
pub trait CacheProvider: Send + Sync {
    /// Get a value from cache
    ///
    /// Returns `Ok(Some(value))` if found, `Ok(None)` if not found, `Err(e)` on error.
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<Vec<u8>>>;

    /// Get multiple values from cache
    ///
    /// Returns a map of found keys to values. Missing keys are not included.
    async fn get_multiple(
        &self,
        namespace: &str,
        keys: &[&str],
    ) -> Result<std::collections::HashMap<String, Vec<u8>>> {
        let mut result = std::collections::HashMap::new();
        for key in keys {
            if let Some(value) = self.get(namespace, key).await? {
                result.insert(key.to_string(), value);
            }
        }
        Ok(result)
    }

    /// Set a value in cache with TTL
    async fn set(&self, namespace: &str, key: &str, value: Vec<u8>, ttl: Duration) -> Result<()>;

    /// Set multiple values in cache
    async fn set_multiple(
        &self,
        namespace: &str,
        items: &[(String, Vec<u8>)],
        ttl: Duration,
    ) -> Result<()> {
        for (key, value) in items {
            self.set(namespace, key, value.clone(), ttl).await?;
        }
        Ok(())
    }

    /// Delete a key from cache
    async fn delete(&self, namespace: &str, key: &str) -> Result<()>;

    /// Delete multiple keys from cache
    async fn delete_multiple(&self, namespace: &str, keys: &[&str]) -> Result<()> {
        for key in keys {
            self.delete(namespace, key).await?;
        }
        Ok(())
    }

    /// Clear cache entries
    ///
    /// - If `namespace` is `Some`, clears only that namespace
    /// - If `namespace` is `None`, clears all namespaces
    async fn clear(&self, namespace: Option<&str>) -> Result<()>;

    /// Check if a key exists in cache
    async fn exists(&self, namespace: &str, key: &str) -> Result<bool> {
        Ok(self.get(namespace, key).await?.is_some())
    }

    /// Get cache statistics for a namespace
    async fn get_stats(&self, namespace: &str) -> Result<CacheStats>;

    /// Get overall cache health
    async fn health_check(&self) -> Result<HealthStatus>;

    /// Get the cache backend type (for logging/debugging)
    fn backend_type(&self) -> String;
}

/// Shared trait object for cache provider
///
/// Use this type instead of concrete implementations:
/// ```ignore
/// fn my_function(cache: SharedCacheProvider) {
///     // Works with any cache provider implementation
/// }
/// ```
pub type SharedCacheProvider = Arc<dyn CacheProvider>;

/// Null cache provider - no-op implementation for testing
/// and when caching is disabled
pub struct NullCacheProvider;

#[async_trait]
impl CacheProvider for NullCacheProvider {
    async fn get(&self, _namespace: &str, _key: &str) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    async fn set(
        &self,
        _namespace: &str,
        _key: &str,
        _value: Vec<u8>,
        _ttl: Duration,
    ) -> Result<()> {
        Ok(())
    }

    async fn delete(&self, _namespace: &str, _key: &str) -> Result<()> {
        Ok(())
    }

    async fn clear(&self, _namespace: Option<&str>) -> Result<()> {
        Ok(())
    }

    async fn get_stats(&self, _namespace: &str) -> Result<CacheStats> {
        Ok(CacheStats::default())
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        Ok(HealthStatus::Healthy)
    }

    fn backend_type(&self) -> String {
        "null".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_null_cache_provider() {
        let cache = NullCacheProvider;

        // All operations should succeed with no-op behavior
        assert!(cache
            .set("test", "key", vec![1, 2, 3], Duration::from_secs(60))
            .await
            .is_ok());
        assert_eq!(cache.get("test", "key").await.unwrap(), None);
        assert!(cache.delete("test", "key").await.is_ok());
        assert!(cache.clear(None).await.is_ok());
        assert_eq!(cache.health_check().await.unwrap(), HealthStatus::Healthy);
        assert_eq!(cache.backend_type(), "null");
    }

    #[tokio::test]
    async fn test_null_cache_exists() {
        let cache = NullCacheProvider;
        assert!(!cache.exists("test", "key").await.unwrap());
    }

    #[tokio::test]
    async fn test_null_cache_stats() {
        let cache = NullCacheProvider;
        let stats = cache.get_stats("test").await.unwrap();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.hits, 0);
    }

    #[tokio::test]
    async fn test_null_cache_multiple_operations() {
        let cache = NullCacheProvider;

        let items = vec![
            ("key1".to_string(), vec![1, 2, 3]),
            ("key2".to_string(), vec![4, 5, 6]),
        ];

        assert!(cache
            .set_multiple("test", &items, Duration::from_secs(60))
            .await
            .is_ok());

        let result = cache.get_multiple("test", &["key1", "key2"]).await.unwrap();
        assert!(result.is_empty());

        assert!(cache
            .delete_multiple("test", &["key1", "key2"])
            .await
            .is_ok());
    }
}
