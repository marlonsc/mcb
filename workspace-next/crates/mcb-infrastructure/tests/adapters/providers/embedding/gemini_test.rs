//! Gemini Embedding Provider Tests

use mcb_domain::ports::EmbeddingProvider;
use mcb_infrastructure::adapters::http_client::test_utils::NullHttpClientPool;
use mcb_infrastructure::adapters::providers::GeminiEmbeddingProvider;
use mcb_infrastructure::constants::EMBEDDING_DIMENSION_GEMINI;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_provider_name() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = GeminiEmbeddingProvider::new(
        "test-key".to_string(),
        None,
        "text-embedding-004".to_string(),
        Duration::from_secs(30),
        http_client,
    );
    assert_eq!(provider.provider_name(), "gemini");
}

#[test]
fn test_dimensions() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = GeminiEmbeddingProvider::new(
        "test-key".to_string(),
        None,
        "text-embedding-004".to_string(),
        Duration::from_secs(30),
        http_client,
    );
    assert_eq!(provider.dimensions(), EMBEDDING_DIMENSION_GEMINI);
}

#[test]
fn test_api_model_name() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = GeminiEmbeddingProvider::new(
        "test-key".to_string(),
        None,
        "models/text-embedding-004".to_string(),
        Duration::from_secs(30),
        http_client,
    );
    assert_eq!(provider.api_model_name(), "text-embedding-004");
}
