//! Tests for snapshot infrastructure

use mcb_infrastructure::infrastructure::snapshot::{NullSnapshotProvider, SnapshotProvider};

#[test]
fn test_null_snapshot_provider_creation() {
    let provider = NullSnapshotProvider::new();
    // Test that provider can be created without panicking
    let _provider: Box<dyn SnapshotProvider> = Box::new(provider);
}

#[test]
fn test_null_snapshot_provider_save() {
    let provider = NullSnapshotProvider::new();

    // Null implementation always succeeds
    let result = provider.save("test-key", b"test-data");
    assert!(result.is_ok());
}

#[test]
fn test_null_snapshot_provider_load() {
    let provider = NullSnapshotProvider::new();

    // Null implementation always returns None
    let result = provider.load("test-key");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_null_snapshot_provider_delete() {
    let provider = NullSnapshotProvider::new();

    // Null implementation always succeeds
    let result = provider.delete("test-key");
    assert!(result.is_ok());
}

#[test]
fn test_null_snapshot_provider_default() {
    let provider = NullSnapshotProvider::default();

    // Test that default implementation works
    let result = provider.save("key", b"data");
    assert!(result.is_ok());
}