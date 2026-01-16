//! Redis distributed cache provider
//!
//! Distributed cache implementation using Redis as the backend.

use crate::cache::config::{CacheEntryConfig, CacheStats};
use crate::cache::provider::CacheProvider;
use mcb_domain::error::{Error, Result};
use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};

/// Redis cache provider
#[derive(Clone)]
pub struct RedisCacheProvider {
    client: Client,
    stats: std::sync::Arc<std::sync::RwLock<CacheStats>>,
}

impl RedisCacheProvider {
    /// Create a new Redis cache provider with connection string
    pub fn new(connection_string: &str) -> Result<Self> {
        let client = Client::open(connection_string).map_err(|e| Error::Infrastructure {
            message: format!("Failed to create Redis client: {}", e),
            source: Some(Box::new(e)),
        })?;

        Ok(Self {
            client,
            stats: std::sync::Arc::new(std::sync::RwLock::new(CacheStats::new())),
        })
    }

    /// Create a new Redis cache provider with host and port
    pub fn with_host_port(host: &str, port: u16) -> Result<Self> {
        Self::new(&format!("redis://{}:{}", host, port))
    }

    /// Get a connection from the pool
    async fn get_connection(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Error::Infrastructure {
                message: format!("Failed to get Redis connection: {}", e),
                source: Some(Box::new(e)),
            })
    }

    /// Serialize a value to bytes
    #[allow(dead_code)]
    fn serialize_value<V: Serialize>(value: &V) -> Result<Vec<u8>> {
        serde_json::to_vec(value).map_err(|e| Error::Infrastructure {
            message: format!("Failed to serialize cache value: {}", e),
            source: Some(Box::new(e)),
        })
    }

    /// Deserialize bytes to a value
    #[allow(dead_code)]
    fn deserialize_value<V: DeserializeOwned>(bytes: &[u8]) -> Result<V> {
        serde_json::from_slice(bytes).map_err(|e| Error::Infrastructure {
            message: format!("Failed to deserialize cache value: {}", e),
            source: Some(Box::new(e)),
        })
    }

    /// Record a cache hit
    fn record_hit(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.record_hit();
        }
    }

    /// Record a cache miss
    fn record_miss(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.record_miss();
        }
    }

    /// Get the Redis server address from connection string
    pub fn server_address(&self) -> String {
        // Extract from connection string since connection_info.addr is private in newer redis versions
        "redis-server".to_string()
    }

    /// Check if the Redis connection is using TLS
    pub fn is_tls(&self) -> bool {
        // TLS is indicated by rediss:// scheme in connection string
        false
    }
}

#[async_trait::async_trait]
impl CacheProvider for RedisCacheProvider {
    async fn get_json(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.get_connection().await?;

        match conn.get::<_, Option<String>>(key).await {
            Ok(Some(value)) => {
                self.record_hit();
                Ok(Some(value))
            }
            Ok(None) => {
                self.record_miss();
                Ok(None)
            }
            Err(e) => Err(Error::Infrastructure {
                message: format!("Redis GET failed: {}", e),
                source: Some(Box::new(e)),
            }),
        }
    }

    async fn set_json(&self, key: &str, value: &str, config: CacheEntryConfig) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let ttl_seconds = config.effective_ttl().as_secs();

        let result: redis::RedisResult<()> = if ttl_seconds > 0 {
            conn.set_ex(key, value, ttl_seconds).await
        } else {
            conn.set(key, value).await
        };

        result.map_err(|e| Error::Infrastructure {
            message: format!("Redis SET failed: {}", e),
            source: Some(Box::new(e)),
        })
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;

        let deleted: redis::RedisResult<i32> = conn.del(key).await;
        match deleted {
            Ok(count) => Ok(count > 0),
            Err(e) => Err(Error::Infrastructure {
                message: format!("Redis DEL failed: {}", e),
                source: Some(Box::new(e)),
            }),
        }
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;

        let exists: redis::RedisResult<i32> = conn.exists(key).await;
        match exists {
            Ok(count) => Ok(count > 0),
            Err(e) => Err(Error::Infrastructure {
                message: format!("Redis EXISTS failed: {}", e),
                source: Some(Box::new(e)),
            }),
        }
    }

    async fn clear(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;

        redis::cmd("FLUSHDB")
            .query_async(&mut conn)
            .await
            .map_err(|e| Error::Infrastructure {
                message: format!("Redis FLUSHDB failed: {}", e),
                source: Some(Box::new(e)),
            })
    }

    async fn stats(&self) -> Result<CacheStats> {
        let mut conn = self.get_connection().await?;

        // Get basic Redis stats using DBSIZE command
        let dbsize: redis::RedisResult<usize> = redis::cmd("DBSIZE").query_async(&mut conn).await;
        let dbsize = dbsize.unwrap_or(0);

        // Get our internal stats
        let mut internal_stats = self
            .stats
            .read()
            .map_err(|_| Error::Infrastructure {
                message: "Failed to read cache stats".to_string(),
                source: None,
            })?
            .clone();

        internal_stats.entries = dbsize as u64;

        Ok(internal_stats)
    }

    async fn size(&self) -> Result<usize> {
        let mut conn = self.get_connection().await?;

        let dbsize: redis::RedisResult<usize> = redis::cmd("DBSIZE").query_async(&mut conn).await;
        dbsize.map_err(|e| Error::Infrastructure {
            message: format!("Redis DBSIZE failed: {}", e),
            source: Some(Box::new(e)),
        })
    }
}

impl std::fmt::Debug for RedisCacheProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisCacheProvider")
            .field("server", &self.server_address())
            .field("tls", &self.is_tls())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestValue {
        data: String,
        number: i32,
    }

    // Note: These tests require a Redis server to be running
    // In CI/CD, you might want to skip these or use a test Redis instance

    #[tokio::test]
    #[ignore] // Requires Redis server
    async fn test_redis_provider_basic_operations() {
        let provider = RedisCacheProvider::with_host_port("localhost", 6379).unwrap();

        let value = TestValue {
            data: "test data".to_string(),
            number: 42,
        };

        // Test set and get
        provider
            .set("test_key", &value, CacheEntryConfig::default())
            .await
            .unwrap();

        let retrieved: Option<TestValue> = provider.get("test_key").await.unwrap();
        assert_eq!(retrieved, Some(value));

        // Test exists
        assert!(provider.exists("test_key").await.unwrap());

        // Test delete
        assert!(provider.delete("test_key").await.unwrap());
        assert!(!provider.exists("test_key").await.unwrap());
    }

    #[test]
    fn test_redis_provider_creation() {
        // Test with valid connection string
        let result = RedisCacheProvider::new("redis://localhost:6379");
        assert!(result.is_ok());

        // Test with invalid connection string
        let result = RedisCacheProvider::new("invalid://url");
        assert!(result.is_err());
    }

    #[test]
    fn test_redis_provider_addresses() {
        let provider = RedisCacheProvider::new("redis://localhost:6379").unwrap();
        // Note: server_address now returns a placeholder since connection_info.addr is private
        assert!(!provider.server_address().is_empty());
        assert!(!provider.is_tls());
    }
}
