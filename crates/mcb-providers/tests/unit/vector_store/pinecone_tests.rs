use mcb_domain::ports::VectorStoreProvider;
use mcb_domain::registry::vector_store::VectorStoreProviderConfig;
use mcb_domain::value_objects::{CollectionId, Embedding};
use mcb_providers::vector_store::pinecone::{PineconeVectorStoreProvider, pinecone_factory};
use rstest::rstest;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

// ── match_to_search_result error propagation ──────────────────────

#[rstest]
#[test]
fn test_match_to_search_result_missing_id_returns_error() {
    let item = serde_json::json!({ "metadata": {} });
    let result = PineconeVectorStoreProvider::match_to_search_result(&item, 0.9);
    let err = result.expect_err("match_to_search_result should fail when id is missing");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("id"),
        "error should mention 'id': {err_msg}"
    );
}

#[rstest]
#[test]
fn test_match_to_search_result_non_string_id_returns_error() {
    let item = serde_json::json!({ "id": 42, "metadata": {} });
    let result = PineconeVectorStoreProvider::match_to_search_result(&item, 0.9);
    let err = result.expect_err("match_to_search_result should fail when id is not a string");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("id"),
        "error should mention 'id': {err_msg}"
    );
}

#[rstest]
#[test]
fn test_match_to_search_result_missing_metadata_returns_error() {
    let item = serde_json::json!({ "id": "vec_123" });
    let result = PineconeVectorStoreProvider::match_to_search_result(&item, 0.9);
    let err = result.expect_err("match_to_search_result should fail when metadata is missing");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("metadata"),
        "error should mention 'metadata': {err_msg}"
    );
}

#[rstest]
#[test]
fn test_match_to_search_result_valid_item_succeeds() {
    let item = serde_json::json!({
        "id": "vec_123",
        "metadata": {
            "file_path": "src/main.rs",
            "content": "fn main() {}",
            "start_line": 1,
            "language": "rust"
        }
    });
    let result = PineconeVectorStoreProvider::match_to_search_result(&item, 0.95);
    let sr = result.expect("match_to_search_result should succeed for valid item");
    assert_eq!(sr.id, "vec_123");
    assert!((sr.score - 0.95).abs() < f64::EPSILON);
}

// ── Factory tests ────────────────────────────────────────────────

#[rstest]
#[test]
fn test_pinecone_factory_missing_api_key_returns_error() {
    let config = VectorStoreProviderConfig {
        provider: "pinecone".to_owned(),
        uri: Some("https://my-index.svc.pinecone.io".to_owned()),
        api_key: None,
        ..Default::default()
    };
    let result = pinecone_factory(&config);
    let err = result
        .map(|_| ())
        .expect_err("pinecone_factory should fail without api_key");
    assert!(
        err.contains("api_key"),
        "error should mention 'api_key': {err}"
    );
}

#[rstest]
#[test]
fn test_pinecone_factory_missing_uri_returns_error() {
    let config = VectorStoreProviderConfig {
        provider: "pinecone".to_owned(),
        api_key: Some("pk-test-key".to_owned()),
        uri: None,
        ..Default::default()
    };
    let result = pinecone_factory(&config);
    let err = result
        .map(|_| ())
        .expect_err("pinecone_factory should fail without uri");
    assert!(err.contains("uri"), "error should mention 'uri': {err}");
}

// ── insert_vectors error propagation ──────────────────────────────

#[rstest]
#[tokio::test]
async fn test_insert_vectors_empty_vectors_returns_error() {
    let provider = PineconeVectorStoreProvider::new(
        "test-key",
        "https://test.pinecone.io",
        Duration::from_secs(5),
        reqwest::Client::new(),
    );
    let collection = CollectionId::from_name("test_collection");
    let vectors: Vec<Embedding> = vec![];
    let metadata: Vec<HashMap<String, Value>> = vec![];

    let result = provider
        .insert_vectors(&collection, &vectors, metadata)
        .await;
    let err = result.expect_err("insert_vectors should fail for empty vectors");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("empty"),
        "error should mention 'empty': {err_msg}"
    );
}

// ── get_vectors_by_ids error propagation ──────────────────────────

#[rstest]
#[tokio::test]
async fn test_get_vectors_by_ids_empty_ids_returns_error() {
    let provider = PineconeVectorStoreProvider::new(
        "test-key",
        "https://test.pinecone.io",
        Duration::from_secs(5),
        reqwest::Client::new(),
    );
    let collection = CollectionId::from_name("test_collection");
    let ids: Vec<String> = vec![];

    let result = provider.get_vectors_by_ids(&collection, &ids).await;
    let err = result.expect_err("get_vectors_by_ids should fail for empty ids");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("empty"),
        "error should mention 'empty': {err_msg}"
    );
}
