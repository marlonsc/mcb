//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Caching infrastructure with TTL and namespaces
//!
//! Provides caching configuration and wiring.
//! Cache provider implementation is delegated to Loco cache via adapter.
//! Types (`CacheEntryConfig`, `CacheStats`, `CacheProvider`) are in mcb-domain.

pub mod config;
pub mod loco_adapter;
pub mod provider;

pub use loco_adapter::LocoCacheAdapter;
