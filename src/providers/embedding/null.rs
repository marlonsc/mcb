//! Null embedding provider for testing and development

use crate::error::Result;
use crate::providers::embedding::EmbeddingProvider;
use crate::types::Embedding;
use async_trait::async_trait;

/// Null embedding provider for testing
/// Returns fixed-size vectors filled with test values
pub struct NullEmbeddingProvider;

impl NullEmbeddingProvider {
    /// Create a new null embedding provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullEmbeddingProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmbeddingProvider for NullEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let embeddings = texts
            .iter()
            .map(|_| Embedding {
                vector: vec![0.1; 128], // Small fixed dimension for testing
                model: "null".to_string(),
                dimensions: 128,
            })
            .collect();

        Ok(embeddings)
    }

    fn model(&self) -> &str {
        "null"
    }

    fn dimensions(&self) -> usize {
        128
    }

    fn max_tokens(&self) -> usize {
        512
    }

    fn provider_name(&self) -> &str {
        "null"
    }
}