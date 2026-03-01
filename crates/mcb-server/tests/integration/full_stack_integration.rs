//! Full-Stack DI Integration Tests
//!
//! Tests end-to-end data flow through the hexagonal architecture:
//! `AppContext` → Provider Handles → Real Providers → Actual Data Operations
//!
//! ## Key Principle
//!
//! These tests validate that:
//! 1. DI container correctly wires all dependencies
//! 2. Provider handles return working providers
//! 3. Data actually flows through the architecture (not mocked)
//! 4. No architectural bypass occurs
//!
//! Uses real providers (Null/InMemory) for deterministic testing.

// Force linkme registration of all providers
extern crate mcb_providers;

use std::sync::Arc;

use mcb_domain::entities::CodeChunk;
use mcb_domain::value_objects::CollectionId;
use rstest::rstest;
use serde_json::json;

use crate::utils::collection::unique_collection;
use crate::utils::test_fixtures::{TEST_EMBEDDING_DIMENSIONS, shared_app_context};

/// Create test code chunks for full-stack testing
fn create_test_chunks() -> Vec<CodeChunk> {
    vec![
        CodeChunk {
            id: "full_stack_chunk_1".to_owned(),
            file_path: "src/config.rs".to_owned(),
            content: "pub struct AppConfig {
    pub host: String,
    pub port: u16,
}"
            .to_owned(),
            start_line: 1,
            end_line: 4,
            language: "rust".to_owned(),
            metadata: json!({"type": "struct", "name": "AppConfig"}),
        },
        CodeChunk {
            id: "full_stack_chunk_2".to_owned(),
            file_path: "src/main.rs".to_owned(),
            content: "#[tokio::main]
async fn main() {
    let config = TestConfigBuilder::new()?.build()?.0;
    run_server(&config).await;
}"
            .to_owned(),
            start_line: 1,
            end_line: 5,
            language: "rust".to_owned(),
            metadata: json!({"type": "function", "name": "main"}),
        },
    ]
}

async fn assert_embedding_batch_shape(
    embedding: &Arc<dyn mcb_domain::ports::EmbeddingProvider>,
    texts: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let embeddings = embedding.embed_batch(texts).await?;

    assert_eq!(embeddings.len(), texts.len());
    let expected_dim = embedding.dimensions();
    for emb in embeddings {
        assert_eq!(emb.dimensions, expected_dim);
        assert_eq!(emb.vector.len(), expected_dim);
        assert!(
            emb.model == "AllMiniLML6V2" || emb.model == "fastembed-test",
            "unexpected model: {}",
            emb.model
        );
    }
    Ok(())
}

// ============================================================================
// Full-Stack Flow Tests
// ============================================================================

#[tokio::test]
async fn test_init_app_creates_working_context() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = shared_app_context()?;

    // Verify embedding handle returns a real provider
    let embedding = ctx.embedding_provider();
    assert_eq!(
        embedding.provider_name(),
        "fastembed",
        "Default should be fastembed (local) provider"
    );
    assert_eq!(
        embedding.dimensions(),
        TEST_EMBEDDING_DIMENSIONS,
        "FastEmbed provider has {TEST_EMBEDDING_DIMENSIONS} dimensions"
    );

    // Verify vector store handle returns a real provider
    let vector_store = ctx.vector_store_provider();
    assert!(
        vector_store.provider_name() == "edgevec",
        "Default should be edgevec vector store"
    );
    Ok(())
}

#[rstest]
#[case(vec!["authentication middleware".to_owned(), "database connection pool".to_owned()])]
#[case(vec!["first text".to_owned()])]
#[case(vec!["second text".to_owned(), "third text".to_owned()])]
#[tokio::test]
async fn test_embedding_generates_real_vectors(
    #[case] texts: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = shared_app_context()?;

    let embedding = ctx.embedding_provider();
    assert_embedding_batch_shape(&embedding, &texts).await?;
    Ok(())
}

#[tokio::test]
async fn test_full_index_and_search_flow() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = shared_app_context()?;

    let embedding = ctx.embedding_provider();
    let vector_store = ctx.vector_store_provider();

    let collection = unique_collection("full-stack");
    let chunks = create_test_chunks();

    // Step 1: Create collection
    vector_store
        .create_collection(
            &CollectionId::from_name(&collection),
            TEST_EMBEDDING_DIMENSIONS,
        )
        .await?;

    // Step 2: Generate embeddings for chunks
    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let embeddings = embedding.embed_batch(&texts).await?;

    // Step 3: Build metadata from chunks
    let metadata: Vec<std::collections::HashMap<String, serde_json::Value>> = chunks
        .iter()
        .map(|chunk| {
            let mut meta = std::collections::HashMap::new();
            meta.insert("id".to_owned(), json!(chunk.id));
            meta.insert("file_path".to_owned(), json!(chunk.file_path));
            meta.insert("content".to_owned(), json!(chunk.content));
            meta.insert("start_line".to_owned(), json!(chunk.start_line));
            meta.insert("end_line".to_owned(), json!(chunk.end_line));
            meta.insert("language".to_owned(), json!(chunk.language));
            meta
        })
        .collect();

    // Step 4: Insert into vector store
    let ids = vector_store
        .insert_vectors(&CollectionId::from_name(&collection), &embeddings, metadata)
        .await?;

    assert_eq!(ids.len(), chunks.len(), "Should insert all chunks");

    // Step 5: Search with a query
    let query_text = "application configuration settings".to_owned();
    let query_embeddings = embedding.embed_batch(&[query_text]).await?;
    let query_vector = &query_embeddings[0].vector;

    let results = vector_store
        .search_similar(&CollectionId::from_name(&collection), query_vector, 5, None)
        .await?;

    // Validate: we should find results (with local FastEmbedProvider)
    assert!(
        !results.is_empty(),
        "Search should return results after indexing real data"
    );

    // Validate result structure
    for result in &results {
        assert!(!result.file_path.is_empty(), "Result should have file path");
        assert!(!result.content.is_empty(), "Result should have content");
        assert!(
            result.score >= 0.0 && result.score <= 1.0,
            "Score should be normalized"
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_provider_accessors_return_same_instance() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = shared_app_context()?;
    // Get embedding provider twice via accessor
    let provider1 = ctx.embedding_provider();
    let provider2 = ctx.embedding_provider();
    // Should be the same Arc (same underlying provider)
    assert!(
        Arc::ptr_eq(&provider1, &provider2),
        "Accessor should return same provider instance"
    );
    Ok(())
}

#[tokio::test]
async fn test_multiple_collections_isolated() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = shared_app_context()?;

    let embedding = ctx.embedding_provider();
    let vector_store = ctx.vector_store_provider();

    // Create two collections
    let collection_a = unique_collection("isolation-a");
    let collection_b = unique_collection("isolation-b");

    vector_store
        .create_collection(
            &CollectionId::from_name(&collection_a),
            TEST_EMBEDDING_DIMENSIONS,
        )
        .await?;
    vector_store
        .create_collection(
            &CollectionId::from_name(&collection_b),
            TEST_EMBEDDING_DIMENSIONS,
        )
        .await?;

    // Insert data only into collection A
    let chunks = create_test_chunks();
    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let embeddings = embedding.embed_batch(&texts).await?;

    let metadata: Vec<std::collections::HashMap<String, serde_json::Value>> = chunks
        .iter()
        .map(|c| {
            let mut m = std::collections::HashMap::new();
            m.insert("content".to_owned(), json!(c.content));
            m.insert("file_path".to_owned(), json!(c.file_path));
            m
        })
        .collect();

    vector_store
        .insert_vectors(
            &CollectionId::from_name(&collection_a),
            &embeddings,
            metadata,
        )
        .await?;

    // Search in both collections
    let query_emb = embedding.embed_batch(&["config".to_owned()]).await?;

    let results_a = vector_store
        .search_similar(
            &CollectionId::from_name(&collection_a),
            &query_emb[0].vector,
            10,
            None,
        )
        .await?;

    let results_b = vector_store
        .search_similar(
            &CollectionId::from_name(&collection_b),
            &query_emb[0].vector,
            10,
            None,
        )
        .await?;

    // Collection A should have results, B should be empty
    assert!(!results_a.is_empty(), "Collection A should have data");
    assert!(
        results_b.is_empty(),
        "Collection B should be empty (isolated)"
    );
    Ok(())
}
