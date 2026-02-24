//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Caching infrastructure with TTL and namespaces.
//!
//! Types (`CacheEntryConfig`, `CacheStats`, `CacheProvider`) are in mcb-domain.

pub mod adapter;
pub mod config;
pub mod provider;

pub use adapter::CacheAdapter;
