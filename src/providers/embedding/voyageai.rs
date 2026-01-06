//! VoyageAI embedding provider implementation

use crate::error::{Error, Result};
use crate::providers::embedding::EmbeddingProvider;
use crate::types::Embedding;
use async_trait::async_trait;

/// VoyageAI embedding provider
pub struct VoyageAIEmbeddingProvider {
    api_key: String,
    model: String,
}

impl VoyageAIEmbeddingProvider {
    /// Create a new VoyageAI embedding provider
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

#[async_trait]
impl EmbeddingProvider for VoyageAIEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // For MVP, return mock embeddings
        // TODO: Implement actual VoyageAI API calls
        let embeddings = texts
            .iter()
            .map(|_| Embedding {
                vector: vec![0.1; 1024], // VoyageAI voyage-code-3 dimensions
                model: self.model.clone(),
                dimensions: 1024,
            })
            .collect();

        Ok(embeddings)
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn dimensions(&self) -> usize {
        1024 // VoyageAI voyage-code-3
    }

    fn max_tokens(&self) -> usize {
        4096
    }

    fn provider_name(&self) -> &str {
        "voyageai"
    }
}