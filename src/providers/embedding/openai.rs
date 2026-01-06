//! OpenAI embedding provider implementation

use crate::error::{Error, Result};
use crate::providers::embedding::EmbeddingProvider;
use crate::types::Embedding;
use async_trait::async_trait;

/// OpenAI embedding provider
pub struct OpenAIEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
}

impl OpenAIEmbeddingProvider {
    /// Create a new OpenAI embedding provider
    pub fn new(api_key: String, base_url: Option<String>, model: String) -> Self {
        Self {
            api_key,
            base_url,
            model,
        }
    }

    /// Get the effective base URL
    fn base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or("https://api.openai.com/v1")
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // For MVP, return mock embeddings
        // TODO: Implement actual OpenAI API calls
        let embeddings = texts
            .iter()
            .map(|_| Embedding {
                vector: vec![0.1; 1536], // OpenAI text-embedding-3-small dimensions
                model: self.model.clone(),
                dimensions: 1536,
            })
            .collect();

        Ok(embeddings)
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn dimensions(&self) -> usize {
        1536 // OpenAI text-embedding-3-small
    }

    fn max_tokens(&self) -> usize {
        8192
    }

    fn provider_name(&self) -> &str {
        "openai"
    }
}