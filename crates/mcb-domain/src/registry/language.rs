//! Language Chunking Provider Registry
//!
//! Auto-registration system for language chunking providers using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at runtime.

use std::collections::HashMap;

/// Configuration for language chunking provider creation
///
/// Contains all configuration options that a language chunking provider might need.
/// Providers should use what they need and ignore the rest.
#[derive(Debug, Clone, Default)]
pub struct LanguageProviderConfig {
    /// Provider name (e.g., "universal", "treesitter", "null")
    pub provider: String,
    /// Maximum chunk size in characters
    pub max_chunk_size: Option<usize>,
    /// Minimum chunk size in characters
    pub min_chunk_size: Option<usize>,
    /// Chunk overlap in characters
    pub overlap: Option<usize>,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(LanguageProviderConfig {
    /// Set the maximum chunk size in characters
    max_chunk_size: with_max_chunk_size(usize),
    /// Set the minimum chunk size in characters
    min_chunk_size: with_min_chunk_size(usize),
    /// Set the chunk overlap in characters
    overlap: with_overlap(usize),
});

crate::impl_registry!(
    provider_trait: crate::ports::providers::LanguageChunkingProvider,
    config_type: LanguageProviderConfig,
    entry_type: LanguageProviderEntry,
    slice_name: LANGUAGE_PROVIDERS,
    resolve_fn: resolve_language_provider,
    list_fn: list_language_providers
);
