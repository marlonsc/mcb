use mcb_domain::ports::VectorStoreProvider;
use mcb_domain::registry::vector_store::VectorStoreProviderConfig;
use mcb_domain::value_objects::{CollectionId, Embedding};
use mcb_providers::vector_store::pinecone::{PineconeVectorStoreProvider, pinecone_factory};
use rstest::{fixture, rstest};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

#[fixture]
fn pinecone_provider() -> PineconeVectorStoreProvider {
    PineconeVectorStoreProvider::new(
        "test-key",
        "https://test.pinecone.io",
        Duration::from_secs(5),
        reqwest::Client::new(),
    )
}

#[fixture]
fn test_collection() -> CollectionId {
    CollectionId::from_name("test_collection")
}

// ---------------------------------------------------------------------------
// match_to_search_result – error cases parametrized with #[case]
// ---------------------------------------------------------------------------

#[rstest]
#[case(serde_json::json!({ "metadata": {} }), "id")]
#[case(serde_json::json!({ "id": 42, "metadata": {} }), "id")]
#[case(serde_json::json!({ "id": "vec_123" }), "metadata")]
fn test_match_to_search_result_error_cases(
    #[case] item: serde_json::Value,
    #[case] expected_substring: &str,
) {
    let result = PineconeVectorStoreProvider::match_to_search_result(&item, 0.9);
    let err = result.expect_err("match_to_search_result should fail");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains(expected_substring),
        "error should mention '{expected_substring}': {err_msg}"
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

// ---------------------------------------------------------------------------
// Factory – missing config fields parametrized with #[case]
// ---------------------------------------------------------------------------

#[rstest]
#[case(
    VectorStoreProviderConfig {
        provider: "pinecone".to_owned(),
        uri: Some("https://my-index.svc.pinecone.io".to_owned()),
        api_key: None,
        ..Default::default()
    },
    "api_key"
)]
#[case(
    VectorStoreProviderConfig {
        provider: "pinecone".to_owned(),
        api_key: Some("pk-test-key".to_owned()),
        uri: None,
        ..Default::default()
    },
    "uri"
)]
fn test_pinecone_factory_missing_config_returns_error(
    #[case] config: VectorStoreProviderConfig,
    #[case] expected_field: &str,
) {
    let result = pinecone_factory(&config);
    let err = result
        .map(|_| ())
        .expect_err("pinecone_factory should fail");
    assert!(
        err.contains(expected_field),
        "error should mention '{expected_field}': {err}"
    );
}

// ---------------------------------------------------------------------------
// insert_vectors / get_vectors_by_ids — empty input errors
// ---------------------------------------------------------------------------

#[rstest]
#[tokio::test]
async fn test_insert_vectors_empty_vectors_returns_error(
    pinecone_provider: PineconeVectorStoreProvider,
    test_collection: CollectionId,
) {
    let vectors: Vec<Embedding> = vec![];
    let metadata: Vec<HashMap<String, Value>> = vec![];

    let result = pinecone_provider
        .insert_vectors(&test_collection, &vectors, metadata)
        .await;
    let err = result.expect_err("insert_vectors should fail for empty vectors");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("empty"),
        "error should mention 'empty': {err_msg}"
    );
}

#[rstest]
#[tokio::test]
async fn test_get_vectors_by_ids_empty_ids_returns_error(
    pinecone_provider: PineconeVectorStoreProvider,
    test_collection: CollectionId,
) {
    let ids: Vec<String> = vec![];

    let result = pinecone_provider
        .get_vectors_by_ids(&test_collection, &ids)
        .await;
    let err = result.expect_err("get_vectors_by_ids should fail for empty ids");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("empty"),
        "error should mention 'empty': {err_msg}"
    );
}
