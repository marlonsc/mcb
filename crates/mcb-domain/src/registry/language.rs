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

impl LanguageProviderConfig {
    /// Create a new config with the given provider name
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            ..Default::default()
        }
    }

    /// Set the maximum chunk size in characters
    pub fn with_max_chunk_size(mut self, size: usize) -> Self {
        self.max_chunk_size = Some(size);
        self
    }

    /// Set the minimum chunk size in characters
    pub fn with_min_chunk_size(mut self, size: usize) -> Self {
        self.min_chunk_size = Some(size);
        self
    }

    /// Set the chunk overlap in characters
    pub fn with_overlap(mut self, overlap: usize) -> Self {
        self.overlap = Some(overlap);
        self
    }

    /// Add extra configuration
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

crate::impl_registry!(
    provider_trait: crate::ports::providers::LanguageChunkingProvider,
    config_type: LanguageProviderConfig,
    entry_type: LanguageProviderEntry,
    slice_name: LANGUAGE_PROVIDERS,
    resolve_fn: resolve_language_provider,
    list_fn: list_language_providers
);
