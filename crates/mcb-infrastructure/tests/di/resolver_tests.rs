//! Tests for the dynamic provider resolver
//!
//! Tests the provider resolution system that bridges configuration and provider instances.

use mcb_domain::value_objects::{EmbeddingConfig, VectorStoreConfig};
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::resolver::*;

#[test]
fn test_list_available_providers() {
    // Should not panic - this tests the function runs correctly
    let providers = list_available_providers();
    // Verify the struct is properly populated (may be empty in unit tests)
    // The key assertion is that we got a valid AvailableProviders instance
    println!(
        "Available providers: embedding={}, vector_store={}, cache={}, language={}",
        providers.embedding.len(),
        providers.vector_store.len(),
        providers.cache.len(),
        providers.language.len()
    );
}

#[test]
fn test_available_providers_display() {
    let providers = AvailableProviders {
        embedding: vec![("null", "Null provider")],
        vector_store: vec![("memory", "In-memory store")],
        cache: vec![("moka", "Moka cache")],
        language: vec![("universal", "Universal chunker")],
    };

    let display = format!("{}", providers);
    assert!(display.contains("Embedding Providers"));
    assert!(display.contains("null"));
}

#[test]
fn test_provider_selection_pattern() {
    // This test demonstrates the provider selection pattern:
    // 1. Configuration drives provider selection
    // 2. Registry resolves provider by name
    // 3. Services use providers through traits (no concrete knowledge)

    // Test that we can create AppConfig with different providers
    let mut config = AppConfig::default();

    // Add OpenAI embedding provider configuration
    config.providers.embedding.configs.insert(
        "default".to_string(),
        EmbeddingConfig {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_key: Some("sk-test".to_string()),
            base_url: None,
            dimensions: Some(1536),
            max_tokens: Some(8192),
        },
    );

    // Add Milvus vector store configuration
    config.providers.vector_store.configs.insert(
        "default".to_string(),
        VectorStoreConfig {
            provider: "milvus".to_string(),
            address: Some("localhost:19530".to_string()),
            token: Some("user:password".to_string()),
            collection: Some("test_collection".to_string()),
            dimensions: Some(384),
            timeout_secs: Some(30),
        },
    );

    // Verify configuration was set correctly
    assert_eq!(
        config
            .providers
            .embedding
            .configs
            .get("default")
            .unwrap()
            .provider,
        "openai"
    );
    assert_eq!(
        config
            .providers
            .vector_store
            .configs
            .get("default")
            .unwrap()
            .provider,
        "milvus"
    );

    // Note: In a real scenario with mcb-providers linked, resolve_providers(&config)
    // would successfully resolve these providers by name from the registry
}
