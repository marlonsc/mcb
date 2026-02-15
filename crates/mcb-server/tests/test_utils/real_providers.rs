//! Real provider fixtures for integration testing
//!
//! Provides factory functions for creating real local providers (InMemory, FastEmbed, Moka)
//! for use in tests that should verify real behavior instead of mocking.
//!
//! Uses a process-wide shared `AppContext` to avoid re-loading the ONNX model
//! (~5-10s) per test.

// Force linkme registration of all providers
extern crate mcb_providers;

use std::sync::Arc;
use std::sync::OnceLock;

use mcb_domain::error::Result;
use mcb_domain::ports::providers::{EmbeddingProvider, VectorStoreProvider};
use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::di::bootstrap::{AppContext, init_app};

/// Process-wide shared AppContext for tests that need real providers.
///
/// Initializes the ONNX model exactly once, then reuses across all tests.
/// The runtime is intentionally leaked so actor tasks survive across tests.
fn shared_app_context() -> &'static AppContext {
    static CTX: OnceLock<AppContext> = OnceLock::new();

    CTX.get_or_init(|| {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().expect("create init runtime");
            let ctx = rt.block_on(async {
                let temp_dir = tempfile::tempdir().expect("create temp dir");
                let temp_path = temp_dir.path().join("mcb-server-shared-test.db");
                std::mem::forget(temp_dir);

                let mut config = ConfigLoader::new().load().expect("load config");
                config.providers.database.configs.insert(
                    "default".to_string(),
                    mcb_infrastructure::config::DatabaseConfig {
                        provider: "sqlite".to_string(),
                        path: Some(temp_path),
                    },
                );
                config.providers.embedding.cache_dir = Some(shared_fastembed_test_cache_dir());
                init_app(config)
                    .await
                    .expect("shared init_app should succeed")
            });
            std::mem::forget(rt);
            ctx
        })
        .join()
        .expect("init thread panicked")
    })
}

fn shared_fastembed_test_cache_dir() -> std::path::PathBuf {
    static CACHE_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();

    CACHE_DIR
        .get_or_init(|| {
            let cache_dir = std::env::var_os("MCB_FASTEMBED_TEST_CACHE_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::env::temp_dir().join("mcb-fastembed-test-cache"));
            std::fs::create_dir_all(&cache_dir).expect("create shared fastembed test cache dir");
            cache_dir
        })
        .clone()
}

/// Get the real EdgeVec vector store provider from the shared context.
pub async fn create_real_vector_store() -> Result<Arc<dyn VectorStoreProvider>> {
    Ok(shared_app_context().vector_store_handle().get())
}

/// Get the real FastEmbed provider from the shared context.
///
/// The ONNX model is loaded once on first access and reused across all tests.
pub async fn create_real_embedding_provider() -> Result<Arc<dyn EmbeddingProvider>> {
    Ok(shared_app_context().embedding_handle().get())
}

/// Get a real FastEmbed provider (model parameter is accepted for API compat).
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

    fn should_run_integration_tests() -> bool {
        // Check for CI environment
        if std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok() {
            // In CI, only run if explicitly enabled
            return std::env::var("MCB_RUN_DOCKER_INTEGRATION_TESTS")
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false);
        }
        // Local: run unless disabled
        std::env::var("MCB_RUN_DOCKER_INTEGRATION_TESTS")
            .map(|v| v != "0" && v != "false")
            .unwrap_or(true)
    }

    #[tokio::test]
    async fn test_real_vector_store_creation() {
        if !should_run_integration_tests() {
            println!("Skipping integration test");
            return;
        }
        let store = create_real_vector_store().await.expect("vector store");
        assert_eq!(store.provider_name(), "edgevec");
    }

    #[tokio::test]
    async fn test_real_vector_store_basic_operations() {
        if !should_run_integration_tests() {
            println!("Skipping integration test");
            return;
        }
        let store = create_real_vector_store().await.expect("vector store");

        // Create collection
        store
            .create_collection(&CollectionId::from_name("test"), 384)
            .await
            .expect("create");

        // Verify collection exists
        let exists = store
            .collection_exists(&CollectionId::from_name("test"))
            .await
            .expect("check exists");
        assert!(exists);

        // Delete collection
        store
            .delete_collection(&CollectionId::from_name("test"))
            .await
            .expect("delete");

        // Verify collection is gone
        let exists = store
            .collection_exists(&CollectionId::from_name("test"))
            .await
            .expect("check exists");
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_real_embedding_provider_creation() {
        if !should_run_integration_tests() {
            println!("Skipping integration test");
            return;
        }
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
        if !should_run_integration_tests() {
            println!("Skipping integration test");
            return;
        }
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
        if !should_run_integration_tests() {
            println!("Skipping integration test");
            return;
        }
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
