//! Full-Stack DI Integration Tests
//!
//! Tests end-to-end data flow through the hexagonal architecture:
//! AppContext → Provider Handles → Real Providers → Actual Data Operations
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
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use rstest::rstest;
use serde_json::json;

fn test_config() -> (AppConfig, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(db_path);
    (config, temp_dir)
}

/// Create test code chunks for full-stack testing
fn create_test_chunks() -> Vec<CodeChunk> {
    vec![
        CodeChunk {
            id: "full_stack_chunk_1".to_string(),
            file_path: "src/config.rs".to_string(),
            content: r#"pub struct AppConfig {
    pub host: String,
    pub port: u16,
}"#
            .to_string(),
            start_line: 1,
            end_line: 4,
            language: "rust".to_string(),
            metadata: json!({"type": "struct", "name": "AppConfig"}),
        },
        CodeChunk {
            id: "full_stack_chunk_2".to_string(),
            file_path: "src/main.rs".to_string(),
            content: r#"#[tokio::main]
async fn main() {
    let config = AppConfig::default();
    run_server(&config).await;
}"#
            .to_string(),
            start_line: 1,
            end_line: 5,
            language: "rust".to_string(),
            metadata: json!({"type": "function", "name": "main"}),
        },
    ]
}

async fn assert_embedding_batch_shape(
    embedding: &Arc<dyn mcb_domain::ports::providers::EmbeddingProvider>,
    texts: &[String],
) {
    let embeddings = embedding
        .embed_batch(texts)
        .await
        .expect("Embedding should work");

    assert_eq!(embeddings.len(), texts.len());
    let expected_dim = embedding.dimensions();
    for emb in embeddings {
        assert_eq!(emb.dimensions, expected_dim);
        assert_eq!(emb.vector.len(), expected_dim);
        assert_eq!(emb.model, "AllMiniLML6V2");
    }
}

// ============================================================================
// Full-Stack Flow Tests
// ============================================================================

#[tokio::test]
async fn test_init_app_creates_working_context() {
    let (config, _temp) = test_config();
    let result = init_app(config).await;

    assert!(
        result.is_ok(),
        "init_app should succeed: {}",
        result
            .as_ref()
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default()
    );

    let ctx = result.expect("Context should be valid");

    // Verify embedding handle returns a real provider
    let embedding = ctx.embedding_handle().get();
    assert_eq!(
        embedding.provider_name(),
        "fastembed",
        "Default should be fastembed (local) provider"
    );
    assert_eq!(
        embedding.dimensions(),
        384,
        "FastEmbed provider has 384 dimensions"
    );

    // Verify vector store handle returns a real provider
    let vector_store = ctx.vector_store_handle().get();
    assert!(
        vector_store.provider_name() == "edgevec",
        "Default should be edgevec vector store"
    );
}

#[rstest]
#[case(vec!["authentication middleware".to_string(), "database connection pool".to_string()])]
#[case(vec!["first text".to_string()])]
#[case(vec!["second text".to_string(), "third text".to_string()])]
#[tokio::test]
async fn test_embedding_generates_real_vectors(#[case] texts: Vec<String>) {
    let (config, _temp) = test_config();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();
    assert_embedding_batch_shape(&embedding, &texts).await;
}

#[tokio::test]
async fn test_full_index_and_search_flow() {
    let (config, _temp) = test_config();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();
    let vector_store = ctx.vector_store_handle().get();

    let collection = "full_stack_test_collection";
    let chunks = create_test_chunks();

    // Step 1: Create collection
    vector_store
        .create_collection(&CollectionId::new(collection), 384)
        .await
        .expect("Collection creation should succeed");

    // Step 2: Generate embeddings for chunks
    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let embeddings = embedding
        .embed_batch(&texts)
        .await
        .expect("Embedding should work");

    // Step 3: Build metadata from chunks
    let metadata: Vec<std::collections::HashMap<String, serde_json::Value>> = chunks
        .iter()
        .map(|chunk| {
            let mut meta = std::collections::HashMap::new();
            meta.insert("id".to_string(), json!(chunk.id));
            meta.insert("file_path".to_string(), json!(chunk.file_path));
            meta.insert("content".to_string(), json!(chunk.content));
            meta.insert("start_line".to_string(), json!(chunk.start_line));
            meta.insert("end_line".to_string(), json!(chunk.end_line));
            meta.insert("language".to_string(), json!(chunk.language));
            meta
        })
        .collect();

    // Step 4: Insert into vector store
    let ids = vector_store
        .insert_vectors(&CollectionId::new(collection), &embeddings, metadata)
        .await
        .expect("Insert should succeed");

    assert_eq!(ids.len(), chunks.len(), "Should insert all chunks");

    // Step 5: Search with a query
    let query_text = "application configuration settings".to_string();
    let query_embeddings = embedding
        .embed_batch(&[query_text])
        .await
        .expect("Query embedding should work");
    let query_vector = &query_embeddings[0].vector;

    let results = vector_store
        .search_similar(&CollectionId::new(collection), query_vector, 5, None)
        .await
        .expect("Search should succeed");

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
}

#[tokio::test]
async fn test_provider_handles_return_same_instance() {
    let (config, _temp) = test_config();
    let ctx = init_app(config).await.expect("init_app should succeed");

    // Get embedding provider twice via handle
    let handle = ctx.embedding_handle();
    let provider1 = handle.get();
    let provider2 = handle.get();

    // Should be the same Arc (same underlying provider)
    assert!(
        Arc::ptr_eq(&provider1, &provider2),
        "Handle should return same provider instance"
    );
}

#[tokio::test]
async fn test_multiple_collections_isolated() {
    let (config, _temp) = test_config();
    let ctx = init_app(config).await.expect("init_app should succeed");

    let embedding = ctx.embedding_handle().get();
    let vector_store = ctx.vector_store_handle().get();

    // Create two collections
    let collection_a = "isolation_test_a";
    let collection_b = "isolation_test_b";

    vector_store
        .create_collection(&CollectionId::new(collection_a), 384)
        .await
        .expect("Create collection A");
    vector_store
        .create_collection(&CollectionId::new(collection_b), 384)
        .await
        .expect("Create collection B");

    // Insert data only into collection A
    let chunks = create_test_chunks();
    let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
    let embeddings = embedding.embed_batch(&texts).await.expect("Embed");

    let metadata: Vec<std::collections::HashMap<String, serde_json::Value>> = chunks
        .iter()
        .map(|c| {
            let mut m = std::collections::HashMap::new();
            m.insert("content".to_string(), json!(c.content));
            m.insert("file_path".to_string(), json!(c.file_path));
            m
        })
        .collect();

    vector_store
        .insert_vectors(&CollectionId::new(collection_a), &embeddings, metadata)
        .await
        .expect("Insert into A");

    // Search in both collections
    let query_emb = embedding
        .embed_batch(&["config".to_string()])
        .await
        .expect("Query embed");

    let results_a = vector_store
        .search_similar(
            &CollectionId::new(collection_a),
            &query_emb[0].vector,
            10,
            None,
        )
        .await
        .expect("Search A");

    let results_b = vector_store
        .search_similar(
            &CollectionId::new(collection_b),
            &query_emb[0].vector,
            10,
            None,
        )
        .await
        .expect("Search B");

    // Collection A should have results, B should be empty
    assert!(!results_a.is_empty(), "Collection A should have data");
    assert!(
        results_b.is_empty(),
        "Collection B should be empty (isolated)"
    );
}
