//! Adapters Module Implementation - External Integrations
//!
//! This module provides adapters for external systems and services.
//! It follows the Shaku strict pattern with no external dependencies.
//!
//! ## Services Provided
//!
//! - Cache provider (with null fallback for testing)
//! - Embedding provider (with null fallback for testing)
//! - Vector store provider (with null fallback for testing)
//! - Language chunking provider (universal implementation)
//!
//! ## Clean Architecture Note
//!
//! Null providers are defined in mcb-infrastructure/adapters/, not imported
//! from mcb-providers. This ensures mcb-infrastructure only depends on
//! traits from mcb-domain, not concrete types from mcb-providers.

use shaku::module;

// Import null providers from mcb-providers crate
use mcb_providers::cache::NullCacheProvider;
use mcb_providers::embedding::NullEmbeddingProvider;
use mcb_providers::language::UniversalLanguageChunkingProvider;
use mcb_providers::vector_store::NullVectorStoreProvider;

// Import traits
use super::traits::AdaptersModule;

module! {
    pub AdaptersModuleImpl: AdaptersModule {
        components = [
            // Null providers (testing fallbacks, overridden in production)
            NullCacheProvider,
            NullEmbeddingProvider,
            NullVectorStoreProvider,
            UniversalLanguageChunkingProvider
        ],
        providers = []
    }
}