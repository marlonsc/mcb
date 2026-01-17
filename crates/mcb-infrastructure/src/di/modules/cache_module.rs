//! Cache Module - Provides caching services
//!
//! This module provides cache provider implementations.
//! Uses MokaCacheProvider as default for production performance.
//! NullCacheProvider available for testing via with_component_override.

use shaku::module;

// Import cache providers - Moka is the production default
use mcb_providers::cache::MokaCacheProvider;

// Import traits
use crate::di::modules::traits::CacheModule;

module! {
    pub CacheModuleImpl: CacheModule {
        components = [
            // Production default: high-performance in-memory cache
            MokaCacheProvider
        ],
        providers = []
    }
}
