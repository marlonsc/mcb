//! Configuration Value Objects
//!
//! Value objects representing configuration for external providers
//! and system settings. These configurations define how the system
//! interacts with external services.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::value_objects::types::{
    CacheProviderKind, EmbeddingProviderKind, VectorStoreProviderKind,
};

const REDACTED: &str = "REDACTED";

/// Value Object: Embedding Provider Configuration
///
/// Configuration for connecting to and using embedding providers.
/// Defines which provider to use and how to authenticate with it.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingConfig {
    /// Provider name (openai, ollama, fastembed, etc.)
    pub provider: EmbeddingProviderKind,
    /// Model identifier specific to the provider
    pub model: String,
    /// API key for cloud providers
    pub api_key: Option<String>,
    /// Custom API endpoint URL
    pub base_url: Option<String>,
    /// Output embedding dimensions
    pub dimensions: Option<usize>,
    /// Maximum input token limit
    pub max_tokens: Option<usize>,
}

impl fmt::Debug for EmbeddingConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmbeddingConfig")
            .field("provider", &self.provider)
            .field("model", &self.model)
            .field("api_key", &self.api_key.as_ref().map(|_| REDACTED))
            .field("base_url", &self.base_url)
            .field("dimensions", &self.dimensions)
            .field("max_tokens", &self.max_tokens)
            .finish()
    }
}

/// Value Object: Vector Store Configuration
///
/// Configuration for connecting to vector storage backends.
/// Defines which storage system to use and connection parameters.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorStoreConfig {
    /// Provider name (edgevec, milvus, qdrant, pinecone)
    pub provider: VectorStoreProviderKind,
    /// Server address for remote providers (e.g., Milvus)
    pub address: Option<String>,
    /// Authentication token for remote providers
    pub token: Option<String>,
    /// Collection name for organizing vectors
    pub collection: Option<String>,
    /// Expected embedding dimensions
    pub dimensions: Option<usize>,
    /// Request timeout in seconds
    pub timeout_secs: Option<u64>,
}

impl fmt::Debug for VectorStoreConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VectorStoreConfig")
            .field("provider", &self.provider)
            .field("address", &self.address)
            .field("token", &self.token.as_ref().map(|_| REDACTED))
            .field("collection", &self.collection)
            .field("dimensions", &self.dimensions)
            .field("timeout_secs", &self.timeout_secs)
            .finish()
    }
}

/// Value Object: Cache Configuration
///
/// Configuration for cache backend providers.
/// Defines which cache provider to use and connection parameters.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheConfig {
    /// Provider name (moka, redis, null)
    pub provider: CacheProviderKind,
    /// Server address for remote providers (e.g., Redis)
    pub address: Option<String>,
    /// Authentication password for remote providers
    pub password: Option<String>,
    /// Database index for Redis
    pub database: Option<u32>,
    /// Maximum cache size in entries
    pub max_size: Option<usize>,
    /// Default TTL in seconds
    pub ttl_secs: Option<u64>,
}

impl fmt::Debug for CacheConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CacheConfig")
            .field("provider", &self.provider)
            .field("address", &self.address)
            .field("password", &self.password.as_ref().map(|_| REDACTED))
            .field("database", &self.database)
            .field("max_size", &self.max_size)
            .field("ttl_secs", &self.ttl_secs)
            .finish()
    }
}

/// Value Object: Synchronization Batch
///
/// Represents a batch of files queued for synchronization/re-indexing.
/// Used by the file watcher daemon to batch file changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncBatch {
    /// Unique batch identifier
    pub id: String,
    /// Repository or collection identifier
    pub collection: String,
    /// List of file paths to process
    pub files: Vec<String>,
    /// Priority level (higher numbers = higher priority)
    pub priority: u8,
    /// Timestamp when batch was created
    pub created_at: i64,
}
