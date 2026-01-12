//! Redis-based cache provider implementation
//!
//! Provides distributed caching using Redis for cluster deployments.
//! Suitable for multi-node deployments or when centralized caching is needed.
//!
//! ## Features
//! - Distributed caching across multiple nodes
//! - TTL support with Redis EXPIRE
//! - Namespace isolation via key prefixing
//! - Connection multiplexing for concurrent access

use crate::domain::error::{Error, Result};
use crate::infrastructure::cache::provider::{CacheProvider, CacheStats, HealthStatus};
use async_trait::async_trait;
use redis::{aio::MultiplexedConnection, Client};
use std::time::Duration;

/// Redis-based cache provider for distributed caching
pub struct RedisCacheProvider {
    client: Client,
}

impl RedisCacheProvider {
    /// Create a new Redis cache provider
    ///
    /// # Arguments
    /// * `url` - Redis connection URL (e.g., "redis://localhost:6379")
    /// * `default_ttl` - Default TTL for keys without explicit TTL
    ///
    /// # Errors
    /// Returns error if unable to connect to Redis or create client
    pub async fn new(url: &str, _default_ttl: Duration) -> Result<Self> {
        tracing::info!("[CACHE] Initializing Redis provider (remote mode): {}", url);

        let client = Client::open(url)
            .map_err(|e| Error::generic(format!("Failed to create Redis client: {}", e)))?;

        // Test connection
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Error::generic(format!("Failed to connect to Redis at {}: {}", url, e)))?;

        // Ping to verify connection
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::generic(format!("Redis PING failed: {}", e)))?;

        if pong != "PONG" {
            return Err(Error::generic("Redis PING did not return PONG".to_string()));
        }

        tracing::info!("[CACHE] Redis connection established");

        Ok(Self { client })
    }

    /// Create a full cache key combining namespace and key
    fn full_key(namespace: &str, key: &str) -> String {
        format!("cache:{}:{}", namespace, key)
    }

    /// Get a Redis connection from the pool
    async fn get_connection(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Error::generic(format!("Failed to get Redis connection: {}", e)))
    }
}

#[async_trait]
impl CacheProvider for RedisCacheProvider {
    async fn get(&self, namespace: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let mut conn = self.get_connection().await?;
        let full_key = Self::full_key(namespace, key);

        let value: Option<Vec<u8>> = redis::cmd("GET")
            .arg(&full_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                tracing::warn!("[CACHE] Redis GET failed for {}: {}", full_key, e);
                Error::generic(format!("Redis GET failed: {}", e))
            })?;

        Ok(value)
    }

    async fn set(&self, namespace: &str, key: &str, value: Vec<u8>, ttl: Duration) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let full_key = Self::full_key(namespace, key);

        redis::cmd("SETEX")
            .arg(&full_key)
            .arg(ttl.as_secs())
            .arg(value)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| {
                tracing::warn!("[CACHE] Redis SET failed for {}: {}", full_key, e);
                Error::generic(format!("Redis SET failed: {}", e))
            })?;

        Ok(())
    }

    async fn delete(&self, namespace: &str, key: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let full_key = Self::full_key(namespace, key);

        redis::cmd("DEL")
            .arg(&full_key)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| {
                tracing::warn!("[CACHE] Redis DEL failed for {}: {}", full_key, e);
                Error::generic(format!("Redis DEL failed: {}", e))
            })?;

        Ok(())
    }

    async fn clear(&self, namespace: Option<&str>) -> Result<()> {
        let mut conn = self.get_connection().await?;

        match namespace {
            Some(ns) => {
                let pattern = format!("cache:{}:*", ns);
                let keys: Vec<String> = redis::cmd("KEYS")
                    .arg(&pattern)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| Error::generic(format!("Redis KEYS failed: {}", e)))?;

                if !keys.is_empty() {
                    redis::cmd("DEL")
                        .arg(&keys)
                        .query_async::<()>(&mut conn)
                        .await
                        .map_err(|e| Error::generic(format!("Redis DEL keys failed: {}", e)))?;
                }
            }
            None => {
                // Clear all cache keys
                let pattern = "cache:*";
                let keys: Vec<String> = redis::cmd("KEYS")
                    .arg(pattern)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| Error::generic(format!("Redis KEYS failed: {}", e)))?;

                if !keys.is_empty() {
                    redis::cmd("DEL")
                        .arg(&keys)
                        .query_async::<()>(&mut conn)
                        .await
                        .map_err(|e| Error::generic(format!("Redis DEL keys failed: {}", e)))?;
                }
            }
        }

        Ok(())
    }

    async fn get_stats(&self, namespace: &str) -> Result<CacheStats> {
        let mut conn = self.get_connection().await?;

        // Count entries in namespace
        let pattern = format!("cache:{}:*", namespace);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::generic(format!("Redis KEYS failed: {}", e)))?;

        // Get Redis info
        let info: String = redis::cmd("INFO")
            .arg("stats")
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::generic(format!("Redis INFO failed: {}", e)))?;

        // Parse basic stats from INFO response
        // This is a simplified implementation - production code might use more detailed parsing
        let mut total_size_bytes = 0u64;
        for line in info.lines() {
            if line.starts_with("used_memory:") {
                if let Ok(size) = line.split(':').nth(1).unwrap_or("0").parse::<u64>() {
                    total_size_bytes = size;
                }
            }
        }

        Ok(CacheStats {
            total_entries: keys.len(),
            total_size_bytes: total_size_bytes as usize,
            hits: 0,   // Redis doesn't expose per-namespace hit count easily
            misses: 0, // Would need custom tracking
            hit_ratio: 0.0,
            evictions: 0, // Would need custom tracking
            avg_access_time_us: 0.0,
        })
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let mut conn = self.get_connection().await?;

        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::generic(format!("Redis PING failed: {}", e)))?;

        if pong == "PONG" {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unhealthy)
        }
    }

    fn backend_type(&self) -> String {
        "redis".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running Redis server
    // Run with: docker run -d -p 6379:6379 redis:latest

    #[tokio::test]
    #[ignore] // Ignore by default - requires Redis
    async fn test_redis_provider_new() {
        let result =
            RedisCacheProvider::new("redis://localhost:6379", Duration::from_secs(60)).await;
        // Will fail if Redis not running, which is expected
        let _ = result;
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_provider_set_and_get() {
        let provider = RedisCacheProvider::new("redis://localhost:6379", Duration::from_secs(60))
            .await
            .unwrap();

        let namespace = "test";
        let key = "test_key";
        let value = vec![1, 2, 3, 4, 5];

        provider
            .set(namespace, key, value.clone(), Duration::from_secs(10))
            .await
            .unwrap();

        let retrieved = provider.get(namespace, key).await.unwrap();
        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_provider_delete() {
        let provider = RedisCacheProvider::new("redis://localhost:6379", Duration::from_secs(60))
            .await
            .unwrap();

        let namespace = "test";
        let key = "test_key";
        let value = vec![1, 2, 3];

        provider
            .set(namespace, key, value, Duration::from_secs(10))
            .await
            .unwrap();

        provider.delete(namespace, key).await.unwrap();

        let retrieved = provider.get(namespace, key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_provider_health_check() {
        let provider = RedisCacheProvider::new("redis://localhost:6379", Duration::from_secs(60))
            .await
            .unwrap();

        let health = provider.health_check().await.unwrap();
        assert_eq!(health, HealthStatus::Healthy);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_provider_backend_type() {
        let provider = RedisCacheProvider::new("redis://localhost:6379", Duration::from_secs(60))
            .await
            .unwrap();
        assert_eq!(provider.backend_type(), "redis".to_string());
    }
}
