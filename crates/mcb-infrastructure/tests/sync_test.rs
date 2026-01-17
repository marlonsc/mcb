//! Tests for sync infrastructure

use mcb_infrastructure::infrastructure::sync::{NullSyncProvider, SyncProvider};

#[test]
fn test_null_sync_provider_creation() {
    let provider = NullSyncProvider::new();
    // Test that provider can be created without panicking
    let _provider: Box<dyn SyncProvider> = Box::new(provider);
}

#[test]
fn test_null_sync_provider_acquire_lock() {
    let provider = NullSyncProvider::new();

    // Null implementation always returns None (no lock)
    let result = provider.acquire_lock("test-resource", None);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_null_sync_provider_release_lock() {
    let provider = NullSyncProvider::new();

    // Null implementation always succeeds
    let result = provider.release_lock("non-existent-lock-id");
    assert!(result.is_ok());
}

#[test]
fn test_null_sync_provider_try_lock() {
    let provider = NullSyncProvider::new();

    // Null implementation always succeeds
    let result = provider.try_lock("test-resource");
    assert!(result.is_ok());
}

#[test]
fn test_null_sync_provider_default() {
    let provider = NullSyncProvider::default();

    // Test that default implementation works
    let result = provider.release_lock("test");
    assert!(result.is_ok());
}