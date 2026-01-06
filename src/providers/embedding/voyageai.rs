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
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings.into_iter().next()
            .ok_or_else(|| Error::embedding("No embedding returned".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let payload = serde_json::json!({
            "input": texts,
            "model": self.model
        });

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.voyageai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::embedding(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::embedding(format!("VoyageAI API error {}: {}", status, error_text)));
        }

        let response_data: serde_json::Value = response.json().await
            .map_err(|e| Error::embedding(format!("Failed to parse response: {}", e)))?;

        let data = response_data["data"].as_array()
            .ok_or_else(|| Error::embedding("Invalid response format: missing data array".to_string()))?;

        if data.len() != texts.len() {
            return Err(Error::embedding(format!("Response data count mismatch: expected {}, got {}", texts.len(), data.len())));
        }

        let embeddings = data.iter().enumerate().map(|(i, item)| {
            let embedding_vec = item["embedding"].as_array()
                .ok_or_else(|| Error::embedding(format!("Invalid embedding format for text {}", i)))?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect::<Vec<f32>>();

            Ok(Embedding {
                vector: embedding_vec,
                model: self.model.clone(),
                dimensions: self.dimensions(),
            })
        }).collect::<Result<Vec<_>>>()?;

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