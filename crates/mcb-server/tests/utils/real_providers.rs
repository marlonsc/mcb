//! Real provider fixtures for integration testing
//!
//! Provides factory functions for creating real local providers (`InMemory`, `FastEmbed`, Moka)
//! for use in tests that should verify real behavior instead of mocking.
//!
//! Uses a process-wide shared `AppContext` to avoid re-loading the ONNX model
//! (~5-10s) per test.

// Force linkme registration of all providers
extern crate mcb_providers;

use std::sync::Arc;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{EmbeddingProvider, VectorStoreProvider};

use super::test_fixtures::{TEST_EMBEDDING_DIMENSIONS, try_shared_app_context};
use rstest::rstest;

/// Get the real `EdgeVec` vector store provider from the shared context.
pub async fn create_real_vector_store() -> Result<Arc<dyn VectorStoreProvider>> {
    let Some(ctx) = try_shared_app_context() else {
        return Err(Error::embedding(
            "Shared AppContext unavailable — FastEmbed model may be missing",
        ));
    };
    Ok(ctx.vector_store_provider())
}

/// Get the real `FastEmbed` provider from the shared context.
///
/// The ONNX model is loaded once on first access and reused across all tests.
pub async fn create_real_embedding_provider() -> Result<Arc<dyn EmbeddingProvider>> {
    let Some(ctx) = try_shared_app_context() else {
        return Err(Error::embedding(
            "Shared AppContext unavailable — FastEmbed model may be missing",
        ));
    };
    Ok(ctx.embedding_provider())
}

/// Get a real `FastEmbed` provider (model parameter is accepted for API compat).
pub async fn create_real_embedding_provider_with_model(
    model: fastembed::EmbeddingModel,
) -> Result<Arc<dyn EmbeddingProvider>> {
    let _ = model;
    create_real_embedding_provider().await
}

#[cfg(test)]
mod tests {
    use mcb_domain::value_objects::CollectionId;

    use mcb_domain::test_collection::unique_collection;

    use super::*;
    use rstest::rstest;

    use mcb_domain::test_service_detection::should_run_docker_integration_tests;

    #[rstest]
    #[tokio::test]
    async fn test_real_vector_store_creation() {
        if !should_run_docker_integration_tests() {
            println!("Skipping integration test");
            return;
        }
        let store = match create_real_vector_store().await {
            Ok(store) => store,
            Err(err) => {
                eprintln!("Skipping integration test (provider unavailable): {err}");
                return;
            }
        };
        assert_eq!(store.provider_name(), "edgevec");
    }

    #[rstest]
    #[tokio::test]
    async fn test_real_vector_store_basic_operations() {
        if !should_run_docker_integration_tests() {
            println!("Skipping integration test");
            return;
        }
        let store = match create_real_vector_store().await {
            Ok(store) => store,
            Err(err) => {
                eprintln!("Skipping integration test (provider unavailable): {err}");
                return;
            }
        };
        let collection = unique_collection("vec-store");

        // Create collection
        store
            .create_collection(
                &CollectionId::from_name(&collection),
                TEST_EMBEDDING_DIMENSIONS,
            )
            .await
            .expect("create");

        // Verify collection exists
        let exists = store
            .collection_exists(&CollectionId::from_name(&collection))
            .await
            .expect("check exists");
        assert!(exists);

        // Delete collection
        store
            .delete_collection(&CollectionId::from_name(&collection))
            .await
            .expect("delete");

        // Verify collection is gone
        let exists = store
            .collection_exists(&CollectionId::from_name(&collection))
            .await
            .expect("check exists");
        assert!(!exists);
    }

    #[rstest]
    #[tokio::test]
    async fn test_real_embedding_provider_creation() {
        if !should_run_docker_integration_tests() {
            println!("Skipping integration test");
            return;
        }
        let provider = match create_real_embedding_provider().await {
            Ok(provider) => provider,
            Err(err) => {
                eprintln!("Skipping integration test (provider unavailable): {err}");
                return;
            }
        };

        let embeddings = provider
            .embed_batch(&["warmup".to_owned()])
            .await
            .expect("fastembed warmup should succeed");
        assert_eq!(provider.provider_name(), "fastembed");
        assert!(provider.dimensions() > 0);
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].vector.len(), TEST_EMBEDDING_DIMENSIONS);
    }

    #[rstest]
    #[tokio::test]
    async fn test_real_embedding_provider_with_model() {
        if !should_run_docker_integration_tests() {
            println!("Skipping integration test");
            return;
        }
        let provider = match create_real_embedding_provider_with_model(
            fastembed::EmbeddingModel::BGESmallENV15,
        )
        .await
        {
            Ok(provider) => provider,
            Err(err) => {
                eprintln!("Skipping integration test (provider unavailable): {err}");
                return;
            }
        };

        let embeddings = provider
            .embed_batch(&["warmup".to_owned()])
            .await
            .expect("fastembed warmup should succeed");
        assert_eq!(provider.provider_name(), "fastembed");
        assert_eq!(embeddings.len(), 1);
        assert_eq!(embeddings[0].vector.len(), TEST_EMBEDDING_DIMENSIONS);
    }

    #[rstest]
    #[tokio::test]
    async fn test_real_embedding_provider_embed_batch() {
        if !should_run_docker_integration_tests() {
            println!("Skipping integration test");
            return;
        }
        let provider = match create_real_embedding_provider().await {
            Ok(provider) => provider,
            Err(err) => {
                eprintln!("Skipping integration test (provider unavailable): {err}");
                return;
            }
        };

        let texts = vec!["hello world".to_owned(), "rust programming".to_owned()];

        let embeddings = provider.embed_batch(&texts).await.expect("embed batch");

        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].dimensions, provider.dimensions());
        assert_eq!(embeddings[1].dimensions, provider.dimensions());
        assert!(embeddings[0].vector.iter().any(|v| *v != 0.0));
        assert!(embeddings[1].vector.iter().any(|v| *v != 0.0));
    }
}
