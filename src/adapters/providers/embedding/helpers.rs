//! Common helpers for embedding providers
//!
//! This module provides shared functionality and patterns used across
//! multiple embedding provider implementations to reduce code duplication.

use crate::domain::error::{Error, Result};
use crate::domain::types::Embedding;
use async_trait::async_trait;

/// Trait that provides default implementations of common embedding provider methods
///
/// This trait is designed to be implemented alongside the EmbeddingProvider trait
/// to provide default implementations of methods that can be shared across multiple
/// provider implementations.
#[async_trait]
pub trait EmbeddingProviderHelper {
    /// Get embeddings for a single text by delegating to embed_batch
    ///
    /// This is a default implementation that wraps a single item in a vector,
    /// calls embed_batch, and extracts the first (and only) result.
    /// Providers implementing this trait can use this as their embed() method.
    async fn embed_single(&self, text: &str) -> Result<Embedding>
    where
        Self: Send + Sync,
    {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| Error::embedding("No embedding returned".to_string()))
    }

    /// Batch embedding method that must be implemented by providers
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>>;
}

/// Common constructor patterns used by embedding providers
///
/// This module provides re-usable patterns for provider initialization.
pub mod constructor {
    use std::time::Duration;

    /// Template for validating and normalizing API keys
    pub fn validate_api_key(api_key: &str) -> String {
        api_key.trim().to_string()
    }

    /// Template for validating and normalizing URLs
    pub fn validate_url(url: Option<String>) -> Option<String> {
        url.map(|u| u.trim().to_string())
    }

    /// Template for default timeout when not specified
    pub fn default_timeout() -> Duration {
        Duration::from_secs(30)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_validate_api_key() {
        assert_eq!(constructor::validate_api_key("  key  "), "key");
        assert_eq!(constructor::validate_api_key("key"), "key");
    }

    #[test]
    fn test_validate_url() {
        assert_eq!(
            constructor::validate_url(Some("  http://example.com  ".to_string())),
            Some("http://example.com".to_string())
        );
        assert_eq!(constructor::validate_url(None), None);
    }

    #[test]
    fn test_default_timeout() {
        assert_eq!(constructor::default_timeout(), Duration::from_secs(30));
    }
}
