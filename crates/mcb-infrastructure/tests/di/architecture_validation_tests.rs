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

use crate::shared_context::shared_app_context;

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
        "Missing expected {} provider '{}'. Registered: {:?}",
        provider_type,
        expected,
        provider_names
    );
}

// ============================================================================
// DI Configuration Consistency
// ============================================================================

#[tokio::test]
async fn test_config_provider_names_match_resolved_providers() {
    let ctx = shared_app_context();
    let embedding = ctx.embedding_handle().get();

    assert_eq!(
        embedding.provider_name(),
        "fastembed",
        "Resolved provider name should match config default"
    );
}

#[tokio::test]
async fn test_handle_based_di_prevents_direct_construction() {
    let ctx = shared_app_context();

    let via_handle_1 = ctx.embedding_handle().get();
    let via_handle_2 = ctx.embedding_handle().get();

    assert!(
        Arc::ptr_eq(&via_handle_1, &via_handle_2),
        "Handle should return same instance (proving DI is used, not direct construction)"
    );
}

#[tokio::test]
async fn test_multiple_handles_reference_same_underlying_provider() {
    let ctx = shared_app_context();

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
    // Test that factory functions create working providers, not just return Ok

    // Embedding provider (local FastEmbed)
    let embedding_config = EmbeddingProviderConfig::new("fastembed");
    let embedding = resolve_embedding_provider(&embedding_config).expect("Should resolve");
    assert_eq!(
        embedding.dimensions(),
        384,
        "FastEmbed should have 384 dimensions"
    );

    // Cache provider (local Moka)
    let cache_config = CacheProviderConfig::new("moka");
    let cache = resolve_cache_provider(&cache_config).expect("Should resolve");
    assert_eq!(cache.provider_name(), "moka", "Should be moka cache");

    // Vector store provider
    let vs_config = VectorStoreProviderConfig::new("edgevec");
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
    let ctx = shared_app_context();

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
            "Embedding provider '{}' has empty description",
            name
        );
        assert!(
            desc.len() > 5,
            "Embedding provider '{}' has too short description: '{}'",
            name,
            desc
        );
    }

    for (name, desc) in list_vector_store_providers() {
        assert!(
            !desc.is_empty(),
            "Vector store provider '{}' has empty description",
            name
        );
    }

    for (name, desc) in list_cache_providers() {
        assert!(
            !desc.is_empty(),
            "Cache provider '{}' has empty description",
            name
        );
    }

    for (name, desc) in list_language_providers() {
        assert!(
            !desc.is_empty(),
            "Language provider '{}' has empty description",
            name
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
        "Should fail for unknown {} provider",
        provider_type
    );
}
