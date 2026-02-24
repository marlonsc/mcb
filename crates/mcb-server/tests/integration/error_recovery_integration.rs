//! Error Recovery Tests
//!
//! Tests that validate graceful error handling and recovery scenarios.
//!
//! ## Key Principle
//!
//! These tests verify:
//! 1. Errors are descriptive and actionable
//! 2. System handles edge cases gracefully
//! 3. Invalid configurations fail fast with clear messages
//! 4. Partial failures don't corrupt state

// Force linkme registration of all providers
extern crate mcb_providers;

use std::sync::Arc;

use mcb_domain::registry::cache::*;
use mcb_domain::registry::embedding::*;
use mcb_domain::registry::language::*;
use mcb_domain::registry::vector_store::*;
use mcb_domain::value_objects::CollectionId;
use rstest::rstest;

use crate::utils::collection::unique_collection;
use crate::utils::test_fixtures::{TEST_EMBEDDING_DIMENSIONS, shared_app_context};

// ============================================================================
// Provider Resolution Error Handling
// ============================================================================

#[rstest]
#[case("embedding")]
#[case("vector_store")]
#[case("cache")]
#[case("language")]
fn test_unknown_provider_error_message(
    #[case] provider_kind: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let result_text = match provider_kind {
        "embedding" => {
            resolve_embedding_provider(&EmbeddingProviderConfig::new("nonexistent_xyz_provider"))
                .err()
                .map(|e| e.to_string())
        }
        "vector_store" => {
            resolve_vector_store_provider(&VectorStoreProviderConfig::new("nonexistent_xyz_store"))
                .err()
                .map(|e| e.to_string())
        }
        "cache" => resolve_cache_provider(&CacheProviderConfig::new("nonexistent_xyz_cache"))
            .err()
            .map(|e| e.to_string()),
        "language" => {
            resolve_language_provider(&LanguageProviderConfig::new("nonexistent_xyz_lang"))
                .err()
                .map(|e| e.to_string())
        }
        _ => None,
    };

    let err_text = result_text.ok_or("Should fail for unknown provider")?;
    assert!(
        err_text.contains("Unknown")
            || err_text.contains("not found")
            || err_text.contains("nonexistent"),
        "Error should mention the issue. Got: {err_text}"
    );
    Ok(())
}

// ============================================================================
// Search on Empty/Missing Collections
// ============================================================================

#[tokio::test]
async fn test_search_empty_collection_returns_empty_not_error()
-> Result<(), Box<dyn std::error::Error>> {
    let ctx = shared_app_context();

    let embedding = ctx.embedding_provider();
    let vector_store = ctx.vector_store_provider();

    let collection = unique_collection("error-empty");

    // Create empty collection
    vector_store
        .create_collection(
            &CollectionId::from_name(&collection),
            TEST_EMBEDDING_DIMENSIONS,
        )
        .await?;

    // Search in empty collection
    let query_embedding = embedding.embed_batch(&["test query".to_owned()]).await?;

    let results = vector_store
        .search_similar(
            &CollectionId::from_name(&collection),
            &query_embedding[0].vector,
            10,
            None,
        )
        .await?;

    assert!(
        results.is_empty(),
        "Empty collection should return empty results"
    );
    Ok(())
}

// ============================================================================
// Configuration Validation
// ============================================================================

#[tokio::test]
async fn test_init_app_with_default_config_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    // Verify the shared (OnceLock) AppContext initialised successfully.
    let _ = shared_app_context();
    Ok(())
}

#[tokio::test]
async fn test_provider_handles_return_valid_instances() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = shared_app_context();

    // All handles should return valid providers
    let embedding = ctx.embedding_provider();
    assert!(
        embedding.dimensions() > 0,
        "Embedding should have positive dimensions"
    );

    let vector_store = ctx.vector_store_provider();
    assert!(
        !vector_store.provider_name().is_empty(),
        "Vector store should have a name"
    );

    let cache = ctx.cache_provider();
    assert!(
        !cache.provider_name().is_empty(),
        "Cache should have a name"
    );
    Ok(())
}

// ============================================================================
// Multiple Operation Error Isolation
// ============================================================================

#[tokio::test]
async fn test_failed_search_doesnt_corrupt_state() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = shared_app_context();

    let embedding = ctx.embedding_provider();
    let vector_store = ctx.vector_store_provider();

    let collection = unique_collection("error-isolation");

    // Create and populate collection
    vector_store
        .create_collection(
            &CollectionId::from_name(&collection),
            TEST_EMBEDDING_DIMENSIONS,
        )
        .await?;

    let embeddings = embedding.embed_batch(&["test data".to_owned()]).await?;

    let metadata = vec![{
        let mut m = std::collections::HashMap::new();
        m.insert("content".to_owned(), serde_json::json!("test"));
        m
    }];

    vector_store
        .insert_vectors(&CollectionId::from_name(&collection), &embeddings, metadata)
        .await?;

    // Try search with wrong dimensions (should fail or handle gracefully)
    let wrong_dim_vector = vec![0.1f32; 100]; // Wrong dimensions

    // This might fail, but shouldn't corrupt the collection
    let _ = vector_store
        .search_similar(
            &CollectionId::from_name(&collection),
            &wrong_dim_vector,
            10,
            None,
        )
        .await;

    // Original search should still work
    let correct_query = embedding.embed_batch(&["test".to_owned()]).await?;

    let results = vector_store
        .search_similar(
            &CollectionId::from_name(&collection),
            &correct_query[0].vector,
            10,
            None,
        )
        .await?;

    assert!(!results.is_empty(), "Collection should not be corrupted");
    Ok(())
}

// ============================================================================
// Registry Robustness
// ============================================================================

#[test]
fn test_list_providers_never_panics() {
    // These should never panic, even if registry is empty
    let embedding_providers = list_embedding_providers();
    let vector_store_providers = list_vector_store_providers();
    let cache_providers = list_cache_providers();
    let language_providers = list_language_providers();

    // With extern crate mcb_providers, none should be empty
    assert!(
        !embedding_providers.is_empty(),
        "Should have embedding providers"
    );
    assert!(
        !vector_store_providers.is_empty(),
        "Should have vector store providers"
    );
    assert!(
        cache_providers.is_empty(),
        "Cache providers are delegated to Loco and should not be linkme-registered"
    );
    assert!(
        !language_providers.is_empty(),
        "Should have language providers"
    );
}

#[test]
fn test_resolve_with_empty_config_values() {
    // Config with empty strings should fail gracefully
    let embedding_config = EmbeddingProviderConfig::new("");
    let result = resolve_embedding_provider(&embedding_config);

    assert!(result.is_err(), "Empty provider name should fail");
}

// ============================================================================
// Concurrent Access Safety
// ============================================================================

#[tokio::test]
async fn test_concurrent_provider_access() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = shared_app_context();
    let provider = ctx.embedding_provider();
    let mut tasks = Vec::new();
    for _ in 0..10 {
        let p = Arc::clone(&provider);
        tasks.push(tokio::spawn(async move { p.dimensions() }));
    }
    for task in tasks {
        let dims = task.await?;
        assert_eq!(
            dims, TEST_EMBEDDING_DIMENSIONS,
            "All accesses should return same dimensions"
        );
    }
    Ok(())
}
