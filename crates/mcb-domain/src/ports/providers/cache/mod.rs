//!
//! **Documentation**: [docs/modules/domain.md](../../../../../../docs/modules/domain.md#provider-ports)
//!
pub(crate) mod config;
mod provider;
mod stats;

pub use config::{CacheEntryConfig, DEFAULT_CACHE_NAMESPACE, DEFAULT_CACHE_TTL_SECS};
pub use provider::CacheProvider;
pub use stats::CacheStats;
