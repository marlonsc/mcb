//! Ollama Embedding Provider Tests

use mcb_domain::ports::EmbeddingProvider;
use mcb_infrastructure::adapters::http_client::test_utils::NullHttpClientPool;
use mcb_infrastructure::adapters::providers::OllamaEmbeddingProvider;
use mcb_infrastructure::constants::EMBEDDING_DIMENSION_OLLAMA_NOMIC;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_provider_name() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = OllamaEmbeddingProvider::new(
        "http://localhost:11434".to_string(),
        "nomic-embed-text".to_string(),
        Duration::from_secs(30),
        http_client,
    );
    assert_eq!(provider.provider_name(), "ollama");
}

#[test]
fn test_dimensions_nomic() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = OllamaEmbeddingProvider::new(
        "http://localhost:11434".to_string(),
        "nomic-embed-text".to_string(),
        Duration::from_secs(30),
        http_client,
    );
    assert_eq!(provider.dimensions(), EMBEDDING_DIMENSION_OLLAMA_NOMIC);
}

#[test]
fn test_model_name() {
    let http_client = Arc::new(NullHttpClientPool::new());
    let provider = OllamaEmbeddingProvider::new(
        "http://localhost:11434".to_string(),
        "all-minilm".to_string(),
        Duration::from_secs(30),
        http_client,
    );
    assert_eq!(provider.model(), "all-minilm");
}
