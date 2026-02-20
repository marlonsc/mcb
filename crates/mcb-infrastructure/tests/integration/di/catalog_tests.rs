//! Catalog DI Integration Tests
//!
//! Tests for the dill Catalog build and service resolution.
//! These tests verify that the `IoC` container initializes correctly
//! with all required providers and services.

use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::di::catalog::build_catalog;
use serial_test::serial;

// Force linkme registration by linking mcb_providers crate
extern crate mcb_providers;

fn configure_offline_embedding(config: &mut mcb_infrastructure::config::AppConfig) {
    config.providers.embedding.provider = Some("openai".to_owned());
    config.providers.embedding.api_key = Some("test-key".to_owned());
    if let Some(default_cfg) = config.providers.embedding.configs.get_mut("default") {
        default_cfg.provider = "openai".to_owned();
        default_cfg.model = "text-embedding-3-small".to_owned();
        default_cfg.api_key = Some("test-key".to_owned());
    }
}

/// Test that catalog builds successfully with default config
#[tokio::test]
#[serial]
async fn test_catalog_builds_with_default_config() {
    let mut config = ConfigLoader::new().load().expect("load config");
    configure_offline_embedding(&mut config);
    let result = build_catalog(config).await;

    assert!(result.is_ok(), "Catalog build failed: {:?}", result.err());
}

/// Test that catalog builds with custom embedding provider config
#[tokio::test]
#[serial]
async fn test_catalog_builds_with_custom_embedding_config() {
    let mut config = ConfigLoader::new().load().expect("load config");
    configure_offline_embedding(&mut config);

    let result = build_catalog(config).await;

    assert!(
        result.is_ok(),
        "Catalog build with custom embedding config failed: {:?}",
        result.err()
    );
}

/// Test that catalog builds with custom vector store config
#[tokio::test]
#[serial]
async fn test_catalog_builds_with_custom_vector_store_config() {
    let mut config = ConfigLoader::new().load().expect("load config");
    configure_offline_embedding(&mut config);
    config.providers.vector_store.provider = Some("edgevec".to_owned());

    let result = build_catalog(config).await;

    assert!(
        result.is_ok(),
        "Catalog build with custom vector store config failed: {:?}",
        result.err()
    );
}

/// Test that catalog builds with custom cache config
#[tokio::test]
#[serial]
async fn test_catalog_builds_with_custom_cache_config() {
    use mcb_infrastructure::config::types::CacheProvider;

    let mut config = ConfigLoader::new().load().expect("load config");
    configure_offline_embedding(&mut config);
    config.system.infrastructure.cache.provider = CacheProvider::Moka;

    let result = build_catalog(config).await;

    assert!(
        result.is_ok(),
        "Catalog build with custom cache config failed: {:?}",
        result.err()
    );
}
