//! Cache provider implementations
//!
//! This module contains concrete implementations of the CacheProvider trait:
//! - Moka: Local in-memory cache (default, single-node)
//! - Redis: Distributed cache (cluster deployments)

pub mod moka;
pub mod redis;

// Public re-exports for external use
#[allow(unused_imports)]
pub use moka::MokaCacheProvider;
#[allow(unused_imports)]
pub use redis::RedisCacheProvider;

// Re-export modules for convenience
#[allow(unused_imports)]
pub use moka as moka_provider;
#[allow(unused_imports)]
pub use redis as redis_provider;
