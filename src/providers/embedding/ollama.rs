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
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings.into_iter().next()
            .ok_or_else(|| Error::embedding("No embedding returned".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Ollama embeddings API expects individual requests
        let mut results = Vec::new();

        for text in texts {
            let payload = serde_json::json!({
                "model": self.model,
                "prompt": text,
                "stream": false
            });

            let client = reqwest::Client::new();
            let response = client
                .post(&format!("{}/api/embeddings", self.base_url.trim_end_matches('/')))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
                .map_err(|e| Error::embedding(format!("HTTP request failed: {}", e)))?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_default();
                return Err(Error::embedding(format!("Ollama API error {}: {}", status, error_text)));
            }

            let response_data: serde_json::Value = response.json().await
                .map_err(|e| Error::embedding(format!("Failed to parse response: {}", e)))?;

            let embedding_vec = response_data["embedding"]
                .as_array()
                .ok_or_else(|| Error::embedding("Invalid response format: missing embedding array".to_string()))?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect::<Vec<f32>>();

            let dimensions = embedding_vec.len();
            results.push(Embedding {
                vector: embedding_vec,
                model: self.model.clone(),
                dimensions,
            });
        }

        Ok(results)
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