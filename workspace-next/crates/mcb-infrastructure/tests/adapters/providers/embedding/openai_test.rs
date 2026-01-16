//! OpenAI Embedding Provider Tests

use mcb_domain::ports::EmbeddingProvider;
use mcb_infrastructure::adapters::http_client::test_utils::NullHttpClientPool;
use mcb_infrastructure::adapters::providers::OpenAIEmbeddingProvider;
use mcb_infrastructure::constants::{
    EMBEDDING_DIMENSION_OPENAI_LARGE, EMBEDDING_DIMENSION_OPENAI_SMALL,
};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_provider_name() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = OpenAIEmbeddingProvider::new(
        "test-key".to_string(),
        None,
        "text-embedding-3-small".to_string(),
        Duration::from_secs(30),
        http_client,
    );
    assert_eq!(provider.provider_name(), "openai");
}

#[test]
fn test_dimensions_small() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = OpenAIEmbeddingProvider::new(
        "test-key".to_string(),
        None,
        "text-embedding-3-small".to_string(),
        Duration::from_secs(30),
        http_client,
    );
    assert_eq!(provider.dimensions(), EMBEDDING_DIMENSION_OPENAI_SMALL);
}

#[test]
fn test_dimensions_large() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = OpenAIEmbeddingProvider::new(
        "test-key".to_string(),
        None,
        "text-embedding-3-large".to_string(),
        Duration::from_secs(30),
        http_client,
    );
    assert_eq!(provider.dimensions(), EMBEDDING_DIMENSION_OPENAI_LARGE);
}

#[test]
fn test_base_url_default() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = OpenAIEmbeddingProvider::new(
        "test-key".to_string(),
        None,
        "text-embedding-3-small".to_string(),
        Duration::from_secs(30),
        http_client,
    );
    assert_eq!(provider.base_url(), "https://api.openai.com/v1");
}

#[test]
fn test_base_url_custom() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = OpenAIEmbeddingProvider::new(
        "test-key".to_string(),
        Some("https://custom.openai.azure.com".to_string()),
        "text-embedding-3-small".to_string(),
        Duration::from_secs(30),
        http_client,
    );
    assert_eq!(provider.base_url(), "https://custom.openai.azure.com");
}
