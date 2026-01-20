//! Null embedding provider for testing and development
//!
//! Provides deterministic, hash-based embeddings for testing purposes.
//! No external dependencies - always works offline.

use async_trait::async_trait;

use mcb_domain::error::Result;
use mcb_domain::ports::providers::EmbeddingProvider;
use mcb_domain::value_objects::Embedding;

use crate::constants::EMBEDDING_DIMENSION_NULL;

/// Null embedding provider for testing
///
/// Returns fixed-size vectors filled with deterministic values based on
/// input text hash. Useful for unit tests and development without requiring
/// an actual embedding service.
///
/// # Example
///
/// ```rust
/// use mcb_providers::embedding::NullEmbeddingProvider;
/// use mcb_domain::ports::providers::EmbeddingProvider;
///
/// let provider = NullEmbeddingProvider::new();
/// assert_eq!(provider.dimensions(), 384);
/// assert_eq!(provider.provider_name(), "null");
/// ```
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
            .map(|text| {
                // Create semantic-like embeddings based on keyword features
                // This enables golden tests to work without real embedding services
                let vector = Self::create_keyword_embedding(text);

                Embedding {
                    vector,
                    model: "null-test".to_string(),
                    dimensions: EMBEDDING_DIMENSION_NULL,
                }
            })
            .collect();

        Ok(embeddings)
    }

    fn dimensions(&self) -> usize {
        EMBEDDING_DIMENSION_NULL
    }

    fn provider_name(&self) -> &str {
        "null"
    }
}

impl NullEmbeddingProvider {
    /// Get the model name for this provider
    pub fn model(&self) -> &str {
        "null"
    }

    /// Get the maximum tokens supported by this provider
    pub fn max_tokens(&self) -> usize {
        512
    }

    /// Create a keyword-based embedding vector for semantic-like matching.
    ///
    /// This method generates distinctive vectors based on domain keywords,
    /// enabling golden tests to find expected files without real embeddings.
    fn create_keyword_embedding(text: &str) -> Vec<f32> {
        let text_lower = text.to_lowercase();
        let mut vector = vec![0.0f32; EMBEDDING_DIMENSION_NULL];

        // Domain keyword features mapped to dimension ranges
        // Each keyword group activates specific dimensions with weighted values
        let keyword_features: &[(&[&str], usize, f32)] = &[
            // Embedding concepts (dimensions 0-30)
            (
                &["embedding", "embed", "vector", "dimension", "similarity"],
                0,
                0.8,
            ),
            // Vector store concepts (dimensions 30-60)
            (
                &[
                    "vector_store",
                    "vectorstore",
                    "collection",
                    "search",
                    "insert",
                ],
                30,
                0.8,
            ),
            // Handler concepts (dimensions 60-90)
            (
                &["handler", "handle", "request", "response", "route"],
                60,
                0.8,
            ),
            // Cache concepts (dimensions 90-120)
            (&["cache", "moka", "redis", "ttl", "expire"], 90, 0.8),
            // DI/Container concepts (dimensions 120-150)
            (
                &["container", "dependency", "inject", "di", "bootstrap"],
                120,
                0.8,
            ),
            // Error concepts (dimensions 150-180)
            (
                &["error", "result", "anyhow", "thiserror", "fail"],
                150,
                0.8,
            ),
            // Chunking concepts (dimensions 180-210)
            (&["chunk", "split", "parse", "ast", "tree_sitter"], 180, 0.8),
            // Provider concepts (dimensions 210-240)
            (&["provider", "trait", "impl", "async_trait"], 210, 0.8),
            // Config concepts (dimensions 240-270)
            (
                &["config", "configuration", "setting", "environment"],
                240,
                0.8,
            ),
            // Test concepts (dimensions 270-300)
            (&["test", "mock", "assert", "expect", "verify"], 270, 0.8),
        ];

        // Activate dimensions based on keyword presence
        for (keywords, start_dim, weight) in keyword_features {
            let matches: usize = keywords
                .iter()
                .map(|kw| if text_lower.contains(kw) { 1 } else { 0 })
                .sum();

            if matches > 0 {
                let activation = weight * (matches as f32 / keywords.len() as f32);
                for i in 0..30 {
                    if start_dim + i < EMBEDDING_DIMENSION_NULL {
                        // Spread activation across the dimension range with variation
                        let variation = ((i as f32) * 0.1).sin() * 0.2;
                        vector[start_dim + i] = (activation + variation).clamp(0.0, 1.0);
                    }
                }
            }
        }

        // Add text-specific variation based on character frequencies
        // Hash normalization divisor scales character sum to small positive value
        const HASH_NORMALIZATION_DIVISOR: f32 = 10_000.0;
        let char_sum: u32 = text.chars().take(100).map(|c| c as u32).sum();
        let hash_base = (char_sum % 1000) as f32 / HASH_NORMALIZATION_DIVISOR;

        // Fill remaining dimensions with hash-based values
        for (i, val) in vector.iter_mut().enumerate() {
            if *val == 0.0 {
                // Add small variation based on position and hash
                *val = hash_base + ((i as f32) * 0.001).sin().abs() * 0.1;
            }
        }

        // Normalize vector for cosine similarity
        let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for val in &mut vector {
                *val /= magnitude;
            }
        }

        vector
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_application::ports::registry::{
    EMBEDDING_PROVIDERS, EmbeddingProviderConfig, EmbeddingProviderEntry,
};
use mcb_domain::ports::providers::EmbeddingProvider as EmbeddingProviderPort;

/// Factory function for creating null embedding provider instances.
fn null_embedding_factory(
    _config: &EmbeddingProviderConfig,
) -> std::result::Result<Arc<dyn EmbeddingProviderPort>, String> {
    Ok(Arc::new(NullEmbeddingProvider::new()))
}

#[linkme::distributed_slice(EMBEDDING_PROVIDERS)]
static NULL_PROVIDER: EmbeddingProviderEntry = EmbeddingProviderEntry {
    name: "null",
    description: "Null provider for testing (deterministic hash-based embeddings)",
    factory: null_embedding_factory,
};
