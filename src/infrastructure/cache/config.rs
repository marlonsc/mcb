//! Cache configuration types
//!
//! Defines configuration structures for the caching system including:
//! - Backend selection (Local Moka or Remote Redis)
//! - Namespace-specific TTL and size settings
//! - Cache entry metadata structures
//!
//! ## Backend Selection
//!
//! Use environment variables to select cache backend:
//! ```bash
//! # Local mode (default)
//! export MCP_CACHE__BACKEND=local
//! export MCP_CACHE__TTL_SECONDS=3600
//!
//! # Remote mode
//! export MCP_CACHE__BACKEND=redis
//! export MCP_REDIS_URL=redis://localhost:6379
//! export MCP_CACHE__TTL_SECONDS=3600
//! ```
//!
//! Or use TOML configuration:
//! ```toml
//! [cache]
//! enabled = true
//! backend = "local"  # or "redis"
//! default_ttl_seconds = 3600
//!
//! [cache.backends.redis]
//! url = "redis://localhost:6379"
//! pool_size = 10
//! ```

use crate::domain::error::Error;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use validator::Validate;

/// Cache backend type with configuration
///
/// Supports pluggable backends for different deployment scenarios:
/// - Local: In-memory Moka cache (single-node, default)
/// - Redis: Distributed cache (cluster deployments)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum CacheBackendConfig {
    /// Local in-memory Moka cache
    #[serde(rename = "local")]
    Local {
        /// Maximum entries for local cache
        max_entries: usize,
        /// Default TTL in seconds
        default_ttl_seconds: u64,
    },
    /// Remote Redis cache
    #[serde(rename = "redis")]
    Redis {
        /// Redis connection URL
        url: String,
        /// Connection pool size
        pool_size: usize,
        /// Default TTL in seconds
        default_ttl_seconds: u64,
    },
}

impl CacheBackendConfig {
    /// Load cache backend configuration from environment variables
    ///
    /// Respects:
    /// - `MCP_CACHE__BACKEND` - "local" or "redis" (default: "local")
    /// - `MCP_CACHE__TTL_SECONDS` - Default TTL (default: 3600)
    /// - `MCP_REDIS_URL` - Redis URL (default: "redis://localhost:6379")
    /// - `MCP_REDIS_POOL_SIZE` - Pool size (default: 10)
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Use local cache
    /// export MCP_CACHE__BACKEND=local
    /// export MCP_CACHE__TTL_SECONDS=7200
    ///
    /// # Use Redis
    /// export MCP_CACHE__BACKEND=redis
    /// export MCP_REDIS_URL=redis://cache-server:6379
    /// export MCP_REDIS_POOL_SIZE=20
    /// ```
    pub fn from_env() -> Self {
        let backend_type =
            std::env::var("MCP_CACHE__BACKEND").unwrap_or_else(|_| "local".to_string());

        let default_ttl_seconds = std::env::var("MCP_CACHE__TTL_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3600);

        match backend_type.to_lowercase().as_str() {
            "redis" => {
                let url = std::env::var("MCP_REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string());
                let pool_size = std::env::var("MCP_REDIS_POOL_SIZE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10);

                CacheBackendConfig::Redis {
                    url,
                    pool_size,
                    default_ttl_seconds,
                }
            }
            _ => CacheBackendConfig::Local {
                max_entries: 10000,
                default_ttl_seconds,
            },
        }
    }

    /// Get the default TTL as a Duration
    pub fn default_ttl(&self) -> Duration {
        let secs = match self {
            CacheBackendConfig::Local {
                default_ttl_seconds,
                ..
            } => *default_ttl_seconds,
            CacheBackendConfig::Redis {
                default_ttl_seconds,
                ..
            } => *default_ttl_seconds,
        };
        Duration::from_secs(secs)
    }

    /// Check if this is local backend
    pub fn is_local(&self) -> bool {
        matches!(self, CacheBackendConfig::Local { .. })
    }

    /// Check if this is Redis backend
    pub fn is_redis(&self) -> bool {
        matches!(self, CacheBackendConfig::Redis { .. })
    }

    /// Get backend type name for logging
    pub fn backend_type(&self) -> &'static str {
        match self {
            CacheBackendConfig::Local { .. } => "local",
            CacheBackendConfig::Redis { .. } => "redis",
        }
    }
}

impl Default for CacheBackendConfig {
    fn default() -> Self {
        CacheBackendConfig::Local {
            max_entries: 10000,
            default_ttl_seconds: 3600,
        }
    }
}

/// Cache configuration
///
/// Main cache system configuration supporting multiple backends.
/// All configuration uses the new CacheBackendConfig enum - legacy fields removed.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CacheConfig {
    /// Whether caching is enabled
    pub enabled: bool,

    /// Cache backend configuration (Moka or Redis)
    pub backend: CacheBackendConfig,

    /// Cache namespaces configuration
    #[validate(nested)]
    pub namespaces: CacheNamespacesConfig,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: CacheBackendConfig::default(),
            namespaces: CacheNamespacesConfig::default(),
        }
    }
}

/// Configuration for different cache namespaces
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CacheNamespacesConfig {
    /// Embedding cache settings
    #[validate(nested)]
    pub embeddings: CacheNamespaceConfig,
    /// Search results cache settings
    #[validate(nested)]
    pub search_results: CacheNamespaceConfig,
    /// Metadata cache settings
    #[validate(nested)]
    pub metadata: CacheNamespaceConfig,
    /// Provider responses cache settings
    #[validate(nested)]
    pub provider_responses: CacheNamespaceConfig,
    /// Sync batches cache settings
    #[validate(nested)]
    pub sync_batches: CacheNamespaceConfig,
}

impl Default for CacheNamespacesConfig {
    fn default() -> Self {
        Self {
            embeddings: CacheNamespaceConfig {
                ttl_seconds: 7200, // 2 hours
                max_entries: 5000,
                compression: true,
            },
            search_results: CacheNamespaceConfig {
                ttl_seconds: 1800, // 30 minutes
                max_entries: 2000,
                compression: false,
            },
            metadata: CacheNamespaceConfig {
                ttl_seconds: 3600, // 1 hour
                max_entries: 1000,
                compression: false,
            },
            provider_responses: CacheNamespaceConfig {
                ttl_seconds: 300, // 5 minutes
                max_entries: 3000,
                compression: true,
            },
            sync_batches: CacheNamespaceConfig {
                ttl_seconds: 86400, // 24 hours
                max_entries: 1000,
                compression: false,
            },
        }
    }
}

/// Configuration for a specific cache namespace
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CacheNamespaceConfig {
    /// TTL for entries in this namespace (seconds)
    #[validate(range(min = 1))]
    pub ttl_seconds: u64,
    /// Maximum number of entries for this namespace
    #[validate(range(min = 1))]
    pub max_entries: usize,
    /// Whether to compress entries
    pub compression: bool,
}

/// Cache entry with metadata
/// Used primarily for Redis serialization to preserve metadata across instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    /// The cached data
    pub data: T,
    /// Timestamp when entry was created
    pub created_at: u64,
    /// Timestamp when entry was last accessed
    pub accessed_at: u64,
    /// Number of times entry was accessed
    pub access_count: u64,
    /// Size of the entry in bytes
    pub size_bytes: usize,
}

/// Cache operation result
#[derive(Debug)]
pub enum CacheResult<T> {
    /// Cache hit with data
    Hit(T),
    /// Cache miss
    Miss,
    /// Cache error
    Error(Error),
}

impl<T> CacheResult<T> {
    /// Check if this is a cache hit
    pub fn is_hit(&self) -> bool {
        matches!(self, CacheResult::Hit(_))
    }

    /// Check if this is a cache miss
    pub fn is_miss(&self) -> bool {
        matches!(self, CacheResult::Miss)
    }

    /// Get the data if it's a hit
    pub fn data(self) -> Option<T> {
        match self {
            CacheResult::Hit(data) => Some(data),
            _ => None,
        }
    }
}
