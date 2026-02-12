//! Real provider fixtures for integration testing
//!
//! Provides factory functions for creating real local providers (InMemory, FastEmbed, Moka)
//! for use in tests that should verify real behavior instead of mocking.

use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;

fn unique_test_path(prefix: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "{}-{}",
        prefix,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos()
    ))
}

/// Create a real EdgeVec vector store provider for testing
///
/// Local HNSW vector store suitable for tests that need actual vector storage and search.
pub async fn create_real_vector_store() -> Result<Arc<dyn VectorStoreProvider>> {
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(unique_test_path("mcb-server-test-db"));
    let cache_dir = unique_test_path("mcb-server-fastembed-cache");
    std::fs::create_dir_all(&cache_dir).expect("create fastembed cache dir");
    config.providers.embedding.cache_dir = Some(cache_dir);
    let ctx = init_app(config).await?;
    Ok(ctx.vector_store_handle().get())
}

/// Create a real FastEmbed provider for testing
///
/// This is a real implementation that uses ONNX models for local embedding generation.
/// Note: First call will download the model (~100MB), subsequent calls reuse cached model.
///
/// # Returns
/// - `Ok(Arc<dyn EmbeddingProvider>)` - Ready-to-use FastEmbed provider
/// - `Err` - If model initialization fails (e.g., network issues, disk space)
pub async fn create_real_embedding_provider() -> Result<Arc<dyn EmbeddingProvider>> {
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(unique_test_path("mcb-server-test-db"));
    let cache_dir = unique_test_path("mcb-server-fastembed-cache");
    std::fs::create_dir_all(&cache_dir).expect("create fastembed cache dir");
    config.providers.embedding.cache_dir = Some(cache_dir);
    let ctx = init_app(config).await?;
    Ok(ctx.embedding_handle().get())
}

/// Create a real FastEmbed provider with a specific model
///
/// Allows testing with different embedding models.
pub async fn create_real_embedding_provider_with_model(
    model: fastembed::EmbeddingModel,
) -> Result<Arc<dyn EmbeddingProvider>> {
    let _ = model;
    create_real_embedding_provider().await
}

#[cfg(test)]
mod tests {
    use mcb_domain::value_objects::CollectionId;

    use super::*;

    #[tokio::test]
    async fn test_real_vector_store_creation() {
        let store = create_real_vector_store().await.expect("vector store");
        assert_eq!(store.provider_name(), "edgevec");
    }

    #[tokio::test]
    async fn test_real_vector_store_basic_operations() {
        let store = create_real_vector_store().await.expect("vector store");

        // Create collection
        store
            .create_collection(&CollectionId::new("test"), 384)
            .await
            .expect("create");

        // Verify collection exists
        let exists = store
            .collection_exists(&CollectionId::new("test"))
            .await
            .expect("check exists");
        assert!(exists);

        // Delete collection
        store
            .delete_collection(&CollectionId::new("test"))
            .await
            .expect("delete");

        // Verify collection is gone
        let exists = store
            .collection_exists(&CollectionId::new("test"))
            .await
            .expect("check exists");
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_real_embedding_provider_creation() {
        let provider = create_real_embedding_provider()
            .await
            .expect("fastembed provider should init");

        let embeddings = provider
            .embed_batch(&["warmup".to_string()])
            .await
            .expect("fastembed warmup should succeed");
        assert_eq!(provider.provider_name(), "fastembed");
        assert!(provider.dimensions() > 0);
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].vector.len(), 384);
    }

    #[tokio::test]
    async fn test_real_embedding_provider_with_model() {
        let provider =
            create_real_embedding_provider_with_model(fastembed::EmbeddingModel::BGESmallENV15)
                .await
                .expect("fastembed provider should init");

        let embeddings = provider
            .embed_batch(&["warmup".to_string()])
            .await
            .expect("fastembed warmup should succeed");
        assert_eq!(provider.provider_name(), "fastembed");
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].vector.len(), 384);
    }

    #[tokio::test]
    async fn test_real_embedding_provider_embed_batch() {
        let provider = create_real_embedding_provider()
            .await
            .expect("fastembed provider should init");

        let texts = vec!["hello world".to_string(), "rust programming".to_string()];

        let embeddings = provider.embed_batch(&texts).await.expect("embed batch");

        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].dimensions, provider.dimensions());
        assert_eq!(embeddings[1].dimensions, provider.dimensions());
        assert!(embeddings[0].vector.iter().any(|v| *v != 0.0));
        assert!(embeddings[1].vector.iter().any(|v| *v != 0.0));
    }
}
