//! Ollama embedding provider implementation

use crate::error::{Error, Result};
use crate::providers::embedding::EmbeddingProvider;
use crate::types::Embedding;
use async_trait::async_trait;

/// Ollama embedding provider
pub struct OllamaEmbeddingProvider {
    base_url: String,
    model: String,
}

impl OllamaEmbeddingProvider {
    /// Create a new Ollama embedding provider
    pub fn new(base_url: String, model: String) -> Self {
        Self { base_url, model }
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // For MVP, return mock embeddings
        // TODO: Implement actual Ollama API calls
        let embeddings = texts
            .iter()
            .map(|_| Embedding {
                vector: vec![0.1; 4096], // Typical Ollama embedding dimensions
                model: self.model.clone(),
                dimensions: 4096,
            })
            .collect();

        Ok(embeddings)
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn dimensions(&self) -> usize {
        4096 // Typical Ollama embedding dimensions
    }

    fn max_tokens(&self) -> usize {
        8192
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }
}