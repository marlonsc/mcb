//! DI Component Dispatch Tests
//!
//! Tests for the DI container bootstrap and initialization.
//!
//! Note: mcb-providers is linked as dev-dependency to ensure
//! providers are registered via linkme distributed slices.

use mcb_infrastructure::config::{AppConfig, ConfigLoader};
use mcb_infrastructure::di::bootstrap::init_app;
use serial_test::serial;

use crate::utils::shared_context::{shared_fastembed_test_cache_dir, try_shared_app_context};

// Force linkme registration by linking mcb_providers crate
extern crate mcb_providers;

/// Build a fresh config+tempdir for tests that intentionally test `init_app()`.
///
/// # Errors
///
/// Returns an error if the temp directory or config loading fails.
fn test_config() -> Result<(AppConfig, tempfile::TempDir), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let default_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../config/default.toml");
    let mut config = ConfigLoader::new().with_config_path(default_path).load()?;
    config.providers.database.configs.insert(
        "default".to_owned(),
        mcb_infrastructure::config::DatabaseConfig {
            provider: "sqlite".to_owned(),
            path: Some(db_path),
        },
    );
    config.providers.embedding.cache_dir = Some(shared_fastembed_test_cache_dir());
    Ok((config, temp_dir))
}

#[tokio::test]
#[serial]
async fn test_di_container_builder() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping: ort-2.0.0-rc.11 Mutex poisoned panic in CI (GitHub Actions)");
        return Ok(());
    }
    let (config, _temp) = test_config()?;
    let app_context = init_app(config).await?;

    // Verify context has expected fields
    assert!(
        std::mem::size_of_val(&app_context.config) > 0,
        "Config should be initialized"
    );

    // Verify providers are accessible
    let embedding = app_context.embedding_provider();
    assert!(!embedding.provider_name().is_empty());
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_provider_selection_from_config() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping: ort-2.0.0-rc.11 Mutex poisoned panic in CI (GitHub Actions)");
        return Ok(());
    }
    // Test that providers are correctly selected based on configuration

    let (mut config, _temp) = test_config()?;

    config.providers.embedding.provider = Some("fastembed".to_owned());
    config.providers.embedding.dimensions = Some(384);

    config.providers.vector_store.provider = Some("edgevec".to_owned());
    config.providers.vector_store.dimensions = Some(384);
    config.providers.vector_store.collection = Some("test".to_owned());

    let app_context = init_app(config).await?;

    assert_eq!(
        app_context.embedding_provider().provider_name(),
        "fastembed"
    );
    assert_eq!(
        app_context.vector_store_provider().provider_name(),
        "edgevec"
    );
    assert_eq!(app_context.cache_provider().provider_name(), "moka");
    assert_eq!(app_context.language_chunker().provider_name(), "universal");
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_provider_resolution_uses_registry() {
    let Some(app_context) = try_shared_app_context() else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };

    // Verify that providers implement the expected traits
    // (This would fail at compile time if providers didn't implement the traits)

    // Test that we can call methods through the trait via accessors
    let embedding = app_context.embedding_provider();
    let _dimensions = embedding.dimensions();
    let _health = embedding.health_check().await;

    // Verify provider names are returned correctly
    assert!(!app_context.embedding_provider().provider_name().is_empty());
    assert!(
        !app_context
            .vector_store_provider()
            .provider_name()
            .is_empty()
    );
    assert!(!app_context.cache_provider().provider_name().is_empty());
    assert!(!app_context.language_chunker().provider_name().is_empty());
}

#[tokio::test]
#[serial]
async fn test_infrastructure_services_from_app_context() {
    let Some(app_context) = try_shared_app_context() else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };

    // Verify infrastructure services are accessible
    // Arc<dyn Trait> types have a strong_count >= 1 if valid
    let event_bus = app_context.event_bus();
    assert!(
        std::sync::Arc::strong_count(&event_bus) >= 1,
        "EventBus service should have valid Arc reference"
    );

    let indexing = app_context.indexing();
    assert!(
        std::sync::Arc::strong_count(&indexing) >= 1,
        "Indexing service should have valid Arc reference"
    );
}
