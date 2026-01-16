//! Null Embedding Provider Tests

use mcb_domain::ports::EmbeddingProvider;
use mcb_infrastructure::adapters::providers::NullEmbeddingProvider;

#[test]
fn test_null_provider_creation() {
    let provider = NullEmbeddingProvider::new();
    assert_eq!(provider.dimensions(), 384);
    assert_eq!(provider.provider_name(), "null");
    assert_eq!(provider.model(), "null");
}

#[test]
fn test_default_trait() {
    let provider = NullEmbeddingProvider::default();
    assert_eq!(provider.dimensions(), 384);
}

#[tokio::test]
async fn test_embed_batch() {
    let provider = NullEmbeddingProvider::new();
    let texts = vec!["hello".to_string(), "world".to_string()];

    let embeddings = provider.embed_batch(&texts).await.unwrap();

    assert_eq!(embeddings.len(), 2);
    assert_eq!(embeddings[0].dimensions, 384);
    assert_eq!(embeddings[0].vector.len(), 384);
    assert_eq!(embeddings[0].model, "null-test");
}

#[tokio::test]
async fn test_embed_batch_empty() {
    let provider = NullEmbeddingProvider::new();
    let texts: Vec<String> = vec![];

    let embeddings = provider.embed_batch(&texts).await.unwrap();

    assert!(embeddings.is_empty());
}

#[tokio::test]
async fn test_deterministic_embeddings() {
    let provider = NullEmbeddingProvider::new();
    let texts = vec!["test".to_string()];

    let embeddings1 = provider.embed_batch(&texts).await.unwrap();
    let embeddings2 = provider.embed_batch(&texts).await.unwrap();

    // Same input should produce same output
    assert_eq!(embeddings1[0].vector, embeddings2[0].vector);
}
