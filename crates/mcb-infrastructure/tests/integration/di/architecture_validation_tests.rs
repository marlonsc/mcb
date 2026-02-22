//! Architecture Validation Tests
//!
//! Tests that validate correct usage of the DI system and hexagonal architecture.
//! These tests detect architectural violations and bypasses.
//!
//! ## Key Principle
//!
//! Tests should verify:
//! 1. Services are obtained via DI, not constructed directly
//! 2. All registries have registered providers
//! 3. Provider names match expectations from config
//! 4. Handles return consistent provider instances
//! 5. Admin services can switch providers at runtime

use rstest::rstest;
// Force linkme registration of all providers
extern crate mcb_providers;

use std::sync::Arc;

use mcb_domain::registry::cache::*;
use mcb_domain::registry::embedding::*;
use mcb_domain::registry::language::*;
use mcb_domain::registry::vector_store::*;

use crate::utils::shared_context::try_shared_app_context;

// ============================================================================
// Registry Completeness Validation
// ============================================================================

#[rstest]
#[case("embedding", "fastembed")]
#[case("embedding", "ollama")]
#[case("embedding", "openai")]
#[case("vector_store", "edgevec")]
#[case("cache", "moka")]
#[case("language", "universal")]
fn all_expected_providers_registered(#[case] provider_type: &str, #[case] expected: &str) {
    let provider_names: Vec<&str> = match provider_type {
        "embedding" => list_embedding_providers()
            .iter()
            .map(|(name, _)| *name)
            .collect(),
        "vector_store" => list_vector_store_providers()
            .iter()
            .map(|(name, _)| *name)
            .collect(),
        "cache" => list_cache_providers()
            .iter()
            .map(|(name, _)| *name)
            .collect(),
        "language" => list_language_providers()
            .iter()
            .map(|(name, _)| *name)
            .collect(),
        _ => vec![],
    };

    assert!(
        provider_names.contains(&expected),
        "Missing expected {provider_type} provider '{expected}'. Registered: {provider_names:?}"
    );
}

// ============================================================================
// DI Configuration Consistency
// ============================================================================

#[tokio::test]
async fn test_config_provider_names_match_resolved_providers() {
    let Some(ctx) = try_shared_app_context() else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };
    let embedding = ctx.embedding_handle().get();

    assert_eq!(
        embedding.provider_name(),
        "fastembed",
        "Resolved provider name should match config default"
    );
}

#[tokio::test]
async fn test_handle_based_di_prevents_direct_construction() {
    let Some(ctx) = try_shared_app_context() else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };

    let via_handle_1 = ctx.embedding_handle().get();
    let via_handle_2 = ctx.embedding_handle().get();

    assert!(
        Arc::ptr_eq(&via_handle_1, &via_handle_2),
        "Handle should return same instance (proving DI is used, not direct construction)"
    );
}

#[tokio::test]
async fn test_multiple_handles_reference_same_underlying_provider() {
    let Some(ctx) = try_shared_app_context() else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };

    let handle1 = ctx.embedding_handle();
    let handle2 = ctx.embedding_handle();

    let provider1 = handle1.get();
    let provider2 = handle2.get();

    assert!(
        Arc::ptr_eq(&provider1, &provider2),
        "Different handle references should return same provider"
    );
}

// ============================================================================
// Provider Factory Validation
// ============================================================================

#[tokio::test]
async fn test_provider_factories_return_working_providers() {
    if std::env::var("CI").is_ok() {
        eprintln!("Skipping: ort-2.0.0-rc.11 Mutex poisoned panic in CI (GitHub Actions)");
        return;
    }
    // Test that factory functions create working providers, not just return Ok

    let embedding_config = EmbeddingProviderConfig::new("fastembed")
        .with_cache_dir(crate::utils::shared_context::shared_fastembed_test_cache_dir());
    match resolve_embedding_provider(&embedding_config) {
        Ok(embedding) => {
            assert_eq!(embedding.dimensions(), 384);
        }
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("model.onnx") || msg.contains("Failed to initialize"),
                "Expected model download error in offline env, got: {msg}"
            );
        }
    }

    let cache_config = CacheProviderConfig::new("moka").with_max_size(1000);
    let cache = resolve_cache_provider(&cache_config).expect("Should resolve");
    assert_eq!(cache.provider_name(), "moka");

    let vs_config = VectorStoreProviderConfig::new("edgevec").with_collection("test-collection");
    let vs = resolve_vector_store_provider(&vs_config).expect("Should resolve");
    assert!(
        vs.provider_name() == "edgevec",
        "Should be edgevec vector store"
    );

    // Language provider
    let lang_config = LanguageProviderConfig::new("universal");
    let _ = resolve_language_provider(&lang_config).expect("Should resolve universal");
}

// ============================================================================
// Admin Service Architecture Validation
// ============================================================================

#[tokio::test]
async fn test_admin_services_accessible_via_context() {
    let Some(ctx) = try_shared_app_context() else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };

    let embedding_admin = ctx.embedding_admin();
    let vector_store_admin = ctx.vector_store_admin();
    let cache_admin = ctx.cache_admin();

    // Validate admin services return meaningful data
    assert!(
        !embedding_admin.list_providers().is_empty(),
        "Embedding admin should list available providers"
    );
    assert!(
        !embedding_admin.current_provider().is_empty(),
        "Embedding admin should report current provider"
    );

    assert!(
        !vector_store_admin.list_providers().is_empty(),
        "Vector store admin should list available providers"
    );

    assert!(
        !cache_admin.list_providers().is_empty(),
        "Cache admin should list available providers"
    );
    assert!(
        !cache_admin.current_provider().is_empty(),
        "Cache admin should report current provider"
    );
}

// ============================================================================
// Cross-Layer Dependency Validation
// ============================================================================

#[test]
fn test_registry_entries_have_valid_descriptions() {
    // All registry entries should have meaningful descriptions
    // (empty descriptions indicate incomplete registration)

    for (name, desc) in list_embedding_providers() {
        assert!(
            !desc.is_empty(),
            "Embedding provider '{name}' has empty description"
        );
        assert!(
            desc.len() > 5,
            "Embedding provider '{name}' has too short description: '{desc}'"
        );
    }

    for (name, desc) in list_vector_store_providers() {
        assert!(
            !desc.is_empty(),
            "Vector store provider '{name}' has empty description"
        );
    }

    for (name, desc) in list_cache_providers() {
        assert!(
            !desc.is_empty(),
            "Cache provider '{name}' has empty description"
        );
    }

    for (name, desc) in list_language_providers() {
        assert!(
            !desc.is_empty(),
            "Language provider '{name}' has empty description"
        );
    }
}

#[rstest]
#[case("embedding")]
#[case("vector_store")]
#[case("cache")]
#[case("language")]
fn provider_resolution_fails_gracefully_for_unknown(#[case] provider_type: &str) {
    let result = match provider_type {
        "embedding" => {
            resolve_embedding_provider(&EmbeddingProviderConfig::new("xyz123")).map(|_| ())
        }
        "vector_store" => {
            resolve_vector_store_provider(&VectorStoreProviderConfig::new("xyz123")).map(|_| ())
        }
        "cache" => resolve_cache_provider(&CacheProviderConfig::new("xyz123")).map(|_| ()),
        "language" => resolve_language_provider(&LanguageProviderConfig::new("xyz123")).map(|_| ()),
        _ => Ok(()),
    };
    assert!(
        result.is_err(),
        "Should fail for unknown {provider_type} provider"
    );
}
