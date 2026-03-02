//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md)
//!
//! Unified retry policy constants for all provider API requests.
//!
//! All providers (embedding, vector store, database) use the same retry
//! behaviour to ensure consistent failure recovery across the system.

/// Default retry count for all provider API requests.
pub const PROVIDER_RETRY_COUNT: usize = 3;

/// Default retry backoff for all provider API requests (milliseconds).
pub const PROVIDER_RETRY_BACKOFF_MS: u64 = 500;
