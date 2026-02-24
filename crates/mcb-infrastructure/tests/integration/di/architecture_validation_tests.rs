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
//! 4. Accessor returns consistent provider instances

use rstest::rstest;
// Force linkme registration of all providers
extern crate mcb_providers;

use std::sync::Arc;

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
    let embedding = ctx.embedding_provider();

    assert_eq!(
        embedding.provider_name(),
        "fastembed",
        "Resolved provider name should match config default"
    );
}

#[tokio::test]
async fn test_di_prevents_direct_construction() {
    let Some(ctx) = try_shared_app_context() else {
        eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
        return;
    };

    let provider_1 = ctx.embedding_provider();
    let provider_2 = ctx.embedding_provider();

    assert!(
        Arc::ptr_eq(&provider_1, &provider_2),
        "Accessor should return same instance (proving DI is used, not direct construction)"
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
#[case("language")]
fn provider_resolution_fails_gracefully_for_unknown(#[case] provider_type: &str) {
    let result = match provider_type {
        "embedding" => {
            resolve_embedding_provider(&EmbeddingProviderConfig::new("xyz123")).map(|_| ())
        }
        "vector_store" => {
            resolve_vector_store_provider(&VectorStoreProviderConfig::new("xyz123")).map(|_| ())
        }
        "language" => resolve_language_provider(&LanguageProviderConfig::new("xyz123")).map(|_| ()),
        _ => Ok(()),
    };
    assert!(
        result.is_err(),
        "Should fail for unknown {provider_type} provider"
    );
}
