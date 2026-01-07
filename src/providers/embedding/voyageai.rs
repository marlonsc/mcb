//! VoyageAI embedding provider implementation

use crate::core::error::{Error, Result};
use crate::core::types::Embedding;
use crate::providers::EmbeddingProvider;
use async_trait::async_trait;

/// VoyageAI embedding provider
pub struct VoyageAIEmbeddingProvider {
    api_key: String,
    base_url: Option<String>,
    model: String,
}

impl VoyageAIEmbeddingProvider {
    /// Create a new VoyageAI embedding provider
    pub fn new(api_key: String, base_url: Option<String>, model: String) -> Self {
        Self {
            api_key,
            base_url,
            model,
        }
    }

    /// Get the effective base URL
    fn effective_base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or("https://api.voyageai.com/v1")
    }
}

#[async_trait]
impl EmbeddingProvider for VoyageAIEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Embedding> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| Error::embedding("No embedding returned".to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Prepare request payload
        let payload = serde_json::json!({
            "input": texts,
            "model": self.model
        });

        // Make HTTP request to VoyageAI
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/embeddings", self.effective_base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::embedding(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::embedding(format!(
                "VoyageAI API error {}: {}",
                status, error_text
            )));
        }

        let response_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::embedding(format!("Failed to parse response: {}", e)))?;

        // Parse embeddings from response
        let data = response_data["data"].as_array().ok_or_else(|| {
            Error::embedding("Invalid response format: missing data array".to_string())
        })?;

        if data.len() != texts.len() {
            return Err(Error::embedding(format!(
                "Response data count mismatch: expected {}, got {}",
                texts.len(),
                data.len()
            )));
        }

        let embeddings = data
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let embedding_vec = item["embedding"]
                    .as_array()
                    .ok_or_else(|| {
                        Error::embedding(format!("Invalid embedding format for text {}", i))
                    })?
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect::<Vec<f32>>();

                Ok(Embedding {
                    vector: embedding_vec,
                    model: self.model.clone(),
                    dimensions: self.dimensions(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(embeddings)
    }

    fn dimensions(&self) -> usize {
        match self.model.as_str() {
            "voyage-code-3" => 1024,
            _ => 1024, // Default for VoyageAI models
        }
    }

    fn provider_name(&self) -> &str {
        "voyageai"
    }
}

impl VoyageAIEmbeddingProvider {
    /// Get the model name for this provider
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the maximum tokens supported by this provider
    pub fn max_tokens(&self) -> usize {
        match self.model.as_str() {
            "voyage-code-3" => 16000,
            _ => 16000, // Default max tokens
        }
    }

    /// Get the API key for this provider
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the base URL for this provider
    pub fn base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or("https://api.voyageai.com/v1")
    }
}
