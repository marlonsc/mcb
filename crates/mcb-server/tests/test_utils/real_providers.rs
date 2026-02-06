//! Real provider fixtures for integration testing
//!
//! Provides factory functions for creating real local providers (InMemory, FastEmbed, Moka)
//! for use in tests that should verify real behavior instead of mocking.

use mcb_domain::error::Result;
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};
use mcb_providers::embedding::FastEmbedProvider;
use mcb_providers::vector_store::{EdgeVecConfig, EdgeVecVectorStoreProvider};
use std::sync::Arc;

/// Create a real EdgeVec vector store provider for testing
///
/// Local HNSW vector store suitable for tests that need actual vector storage and search.
pub fn create_real_vector_store() -> Arc<dyn VectorStoreProvider> {
    Arc::new(
        EdgeVecVectorStoreProvider::new(EdgeVecConfig {
            dimensions: 384,
            ..Default::default()
        })
        .expect("EdgeVec init for tests"),
    )
}

/// Create a real FastEmbed provider for testing
///
/// This is a real implementation that uses ONNX models for local embedding generation.
/// Note: First call will download the model (~100MB), subsequent calls reuse cached model.
///
/// # Returns
/// - `Ok(Arc<dyn EmbeddingProvider>)` - Ready-to-use FastEmbed provider
/// - `Err` - If model initialization fails (e.g., network issues, disk space)
pub fn create_real_embedding_provider() -> Result<Arc<dyn EmbeddingProvider>> {
    let provider = FastEmbedProvider::new()?;
    Ok(Arc::new(provider))
}

/// Create a real FastEmbed provider with a specific model
///
/// Allows testing with different embedding models.
pub fn create_real_embedding_provider_with_model(
    model: fastembed::EmbeddingModel,
) -> Result<Arc<dyn EmbeddingProvider>> {
    let provider = FastEmbedProvider::with_model(model)?;
    Ok(Arc::new(provider))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_real_vector_store_creation() {
        let store = create_real_vector_store();
        assert_eq!(store.provider_name(), "edgevec");
    }

    #[tokio::test]
    async fn test_real_vector_store_basic_operations() {
        let store = create_real_vector_store();

        // Create collection
        store.create_collection("test", 384).await.expect("create");

        // Verify collection exists
        let exists = store.collection_exists("test").await.expect("check exists");
        assert!(exists);

        // Delete collection
        store.delete_collection("test").await.expect("delete");

        // Verify collection is gone
        let exists = store.collection_exists("test").await.expect("check exists");
        assert!(!exists);
    }

    #[tokio::test]
    #[ignore] // Slow test - downloads ~100MB model on first run
    async fn test_real_embedding_provider_creation() {
        let provider = create_real_embedding_provider().expect("create provider");
        assert_eq!(provider.provider_name(), "fastembed");
        assert!(provider.dimensions() > 0);
    }

    #[tokio::test]
    #[ignore] // Slow test - downloads ~100MB model on first run
    async fn test_real_embedding_provider_embed_batch() {
        let provider = create_real_embedding_provider().expect("create provider");

        let texts = vec!["hello world".to_string(), "rust programming".to_string()];

        let embeddings = provider.embed_batch(&texts).await.expect("embed batch");

        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].dimensions, provider.dimensions());
        assert_eq!(embeddings[1].dimensions, provider.dimensions());
    }
}
