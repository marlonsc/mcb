//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Retry configuration constants for provider API requests.

/// Default retry count for embedding API requests.
pub const EMBEDDING_RETRY_COUNT: usize = 3;

/// Default retry backoff for embedding API requests (milliseconds).
pub const EMBEDDING_RETRY_BACKOFF_MS: u64 = 500;

/// Default retry count for vector store API requests.
pub const VECTOR_STORE_RETRY_COUNT: usize = 2;

/// Default retry backoff for vector store API requests (seconds).
pub const VECTOR_STORE_RETRY_BACKOFF_SECS: u64 = 1;
