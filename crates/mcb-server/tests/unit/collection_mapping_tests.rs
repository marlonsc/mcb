//! Tests for collection name mapping
//!
//! Note: These tests use the public API which persists mappings to disk.
//! Use unique collection names to avoid test interference.

use mcb_server::collection_mapping;
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a unique test collection name to avoid conflicts
fn unique_name(prefix: &str) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{}", prefix, ts)
}

#[test]
fn test_generate_milvus_name_format() {
    // Test via the public API - map_collection_name returns Milvus-compatible names
    let input = unique_name("test-collection-unit");
    let result = collection_mapping::map_collection_name(&input).unwrap();
    // Should have underscores instead of hyphens and a timestamp suffix
    assert!(result.as_str().contains("test_collection_unit_"));
    assert!(!result.as_str().contains('-')); // No hyphens in output
}

#[test]
fn test_generate_milvus_name_lowercase() {
    let input = unique_name("TestUpperCase");
    let result = collection_mapping::map_collection_name(&input).unwrap();
    // Should be lowercase
    assert!(!result.as_str().contains(|c: char| c.is_uppercase()));
}

#[test]
fn test_generate_milvus_name_hyphens_converted() {
    let input = unique_name("my-project-hyphens");
    let result = collection_mapping::map_collection_name(&input).unwrap();
    // Hyphens should be converted to underscores
    assert!(!result.as_str().contains('-'));
    assert!(result.as_str().contains('_'));
}

#[test]
fn test_map_collection_name_persistence() {
    // Same input should return same output (persistent mapping)
    let input = unique_name("persistent-test");
    let name1 = collection_mapping::map_collection_name(&input).unwrap();
    let name2 = collection_mapping::map_collection_name(&input).unwrap();
    assert_eq!(name1, name2);
}
