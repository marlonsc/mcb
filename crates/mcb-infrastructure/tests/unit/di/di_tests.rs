//! Unit tests for DI resolver module.

use mcb_infrastructure::di::{AvailableProviders, list_available_providers};
use rstest::rstest;

#[rstest]
fn test_list_available_providers() {
    // Verifies the function is callable and returns valid data
    let providers = list_available_providers();

    // Verify the AvailableProviders struct is valid
    // Providers may be empty in unit tests since mcb-providers isn't linked
    // but the Display implementation should work on any state
    let display = format!("{providers}");
    assert!(
        display.contains("Embedding Providers"),
        "Display should include Embedding Providers section"
    );
}

#[rstest]
fn test_available_providers_display() {
    let providers = AvailableProviders {
        embedding: vec![("fastembed", "FastEmbed local provider")],
        vector_store: vec![("edgevec", "EdgeVec HNSW store")],
        cache: vec![("moka", "Moka cache")],
        language: vec![("universal", "Universal chunker")],
    };

    let display = format!("{providers}");
    assert!(display.contains("Embedding Providers"));
    assert!(display.contains("fastembed"));
    assert!(display.contains("Vector Store Providers"));
    assert!(display.contains("edgevec"));
    assert!(display.contains("Cache Providers"));
    assert!(display.contains("moka"));
    assert!(display.contains("Language Providers"));
    assert!(display.contains("universal"));
}

#[rstest]
fn test_available_providers_empty() {
    let providers = AvailableProviders {
        embedding: vec![],
        vector_store: vec![],
        cache: vec![],
        language: vec![],
    };

    let display = format!("{providers}");
    // Even with empty providers, section headers should be present
    assert!(display.contains("Embedding Providers"));
}
