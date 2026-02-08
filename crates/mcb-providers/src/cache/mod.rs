//! Cache Provider Implementations
//!
//! Provides caching backends for embedding and search result caching.
//!
//! ## Available Providers
//!
//! | Provider | Type | Description |
//! |----------|------|-------------|
//! | MokaCacheProvider | Local | In-memory cache (high performance) |
//! | RedisCacheProvider | Distributed | Redis-backed for multi-instance |
//!
//! ## Provider Selection Guide
//!
//! - **Development/Testing**: Use `MokaCacheProvider` for local in-memory caching
//! - **Single Instance**: Use `MokaCacheProvider` for high performance
//! - **Multi Instance**: Use `RedisCacheProvider` for distributed caching

pub mod moka;
pub mod redis;

// Re-export for convenience
// Re-export domain types used by cache providers
pub use mcb_domain::ports::providers::cache::{CacheEntryConfig, CacheStats};
pub use moka::MokaCacheProvider;
pub use redis::RedisCacheProvider;
