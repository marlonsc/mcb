//! Unit tests for configuration value objects.

use mcb_domain::{EmbeddingConfig, VectorStoreConfig};
use rstest::*;

fn make_embedding_config(
    provider: &str,
    model: &str,
    api_key: Option<&str>,
    base_url: Option<&str>,
    dimensions: Option<usize>,
    max_tokens: Option<usize>,
) -> EmbeddingConfig {
    EmbeddingConfig {
        provider: provider.to_string(),
        model: model.to_string(),
        api_key: api_key.map(str::to_string),
        base_url: base_url.map(str::to_string),
        dimensions,
        max_tokens,
    }
}

fn make_vector_store_config(
    provider: &str,
    address: Option<&str>,
    token: Option<&str>,
    collection: Option<&str>,
    dimensions: Option<usize>,
    timeout_secs: Option<u64>,
) -> VectorStoreConfig {
    VectorStoreConfig {
        provider: provider.to_string(),
        address: address.map(str::to_string),
        token: token.map(str::to_string),
        collection: collection.map(str::to_string),
        dimensions,
        timeout_secs,
    }
}

#[test]
fn test_embedding_config_creation() {
    let config = EmbeddingConfig {
        provider: "openai".to_string(),
        model: "text-embedding-ada-002".to_string(),
        api_key: Some("sk-...".to_string()),
        base_url: None,
        dimensions: Some(1536),
        max_tokens: Some(8191),
    };

    assert_eq!(config.provider, "openai");
    assert_eq!(config.model, "text-embedding-ada-002");
    assert_eq!(config.api_key, Some("sk-...".to_string()));
    assert_eq!(config.base_url, None);
    assert_eq!(config.dimensions, Some(1536));
    assert_eq!(config.max_tokens, Some(8191));
}

#[rstest]
#[case(
    "openai",
    "text-embedding-ada-002",
    Some("sk-..."),
    None,
    Some(1536),
    Some(8191)
)]
#[case("fastembed", "default-model", None, None, None, None)]
#[case(
    "ollama",
    "llama2",
    None,
    Some("http://localhost:11434"),
    Some(4096),
    Some(4096)
)]
fn embedding_config_variants(
    #[case] provider: &str,
    #[case] model: &str,
    #[case] api_key: Option<&str>,
    #[case] base_url: Option<&str>,
    #[case] dimensions: Option<usize>,
    #[case] max_tokens: Option<usize>,
) {
    let config = make_embedding_config(provider, model, api_key, base_url, dimensions, max_tokens);

    assert_eq!(config.provider, provider);
    assert_eq!(config.model, model);
    assert_eq!(config.api_key.as_deref(), api_key);
    assert_eq!(config.base_url.as_deref(), base_url);
    assert_eq!(config.dimensions, dimensions);
    assert_eq!(config.max_tokens, max_tokens);
}

#[test]
fn test_vector_store_config_creation() {
    let config = VectorStoreConfig {
        provider: "qdrant".to_string(),
        address: Some("localhost:6334".to_string()),
        token: None,
        collection: Some("my-collection".to_string()),
        dimensions: Some(1536),
        timeout_secs: Some(30),
    };

    assert_eq!(config.provider, "qdrant");
    assert_eq!(config.address, Some("localhost:6334".to_string()));
    assert_eq!(config.token, None);
    assert_eq!(config.collection, Some("my-collection".to_string()));
    assert_eq!(config.dimensions, Some(1536));
    assert_eq!(config.timeout_secs, Some(30));
}

#[rstest]
#[case(
    "qdrant",
    Some("localhost:6334"),
    None,
    Some("my-collection"),
    Some(1536),
    Some(30)
)]
#[case("edgevec", None, None, Some("local-vectors"), Some(384), None)]
#[case(
    "milvus",
    Some("http://localhost:19530"),
    Some("root:Milvus"),
    Some("embeddings"),
    Some(768),
    Some(60)
)]
fn vector_store_config_variants(
    #[case] provider: &str,
    #[case] address: Option<&str>,
    #[case] token: Option<&str>,
    #[case] collection: Option<&str>,
    #[case] dimensions: Option<usize>,
    #[case] timeout_secs: Option<u64>,
) {
    let config = make_vector_store_config(
        provider,
        address,
        token,
        collection,
        dimensions,
        timeout_secs,
    );

    assert_eq!(config.provider, provider);
    assert_eq!(config.address.as_deref(), address);
    assert_eq!(config.token.as_deref(), token);
    assert_eq!(config.collection.as_deref(), collection);
    assert_eq!(config.dimensions, dimensions);
    assert_eq!(config.timeout_secs, timeout_secs);
}
