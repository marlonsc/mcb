//! VoyageAI Embedding Provider Tests

use mcb_domain::ports::EmbeddingProvider;
use mcb_infrastructure::adapters::http_client::test_utils::NullHttpClientPool;
use mcb_infrastructure::adapters::providers::VoyageAIEmbeddingProvider;
use mcb_infrastructure::constants::EMBEDDING_DIMENSION_VOYAGEAI_CODE;
use std::sync::Arc;

#[test]
fn test_provider_name() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = VoyageAIEmbeddingProvider::new(
        "test-key".to_string(),
        None,
        "voyage-code-3".to_string(),
        http_client,
    );
    assert_eq!(provider.provider_name(), "voyageai");
}

#[test]
fn test_dimensions() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = VoyageAIEmbeddingProvider::new(
        "test-key".to_string(),
        None,
        "voyage-code-3".to_string(),
        http_client,
    );
    assert_eq!(provider.dimensions(), EMBEDDING_DIMENSION_VOYAGEAI_CODE);
}

#[test]
fn test_base_url_default() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = VoyageAIEmbeddingProvider::new(
        "test-key".to_string(),
        None,
        "voyage-code-3".to_string(),
        http_client,
    );
    assert_eq!(provider.base_url(), "https://api.voyageai.com/v1");
}
