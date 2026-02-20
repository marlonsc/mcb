//! Provides file hash repository domain definitions.
use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;

use crate::error::Result;

/// Repository for tracking file content hashes and changes
#[async_trait]
pub trait FileHashRepository: Send + Sync {
    /// Get hash for a file (returns None if not found or tombstoned)
    async fn get_hash(&self, collection: &str, file_path: &str) -> Result<Option<String>>;

    /// Check if file has changed (returns true if new or hash differs)
    async fn has_changed(
        &self,
        collection: &str,
        file_path: &str,
        current_hash: &str,
    ) -> Result<bool>;

    /// Upsert hash for a file (insert or update)
    async fn upsert_hash(&self, collection: &str, file_path: &str, hash: &str) -> Result<()>;

    /// Mark a file as deleted (tombstone)
    async fn mark_deleted(&self, collection: &str, file_path: &str) -> Result<()>;

    /// Get all active file paths for a collection
    async fn get_indexed_files(&self, collection: &str) -> Result<Vec<String>>;

    /// Cleanup tombstones older than default TTL
    async fn cleanup_tombstones(&self) -> Result<u64>;

    /// Cleanup tombstones with custom TTL
    async fn cleanup_tombstones_with_ttl(&self, ttl: Duration) -> Result<u64>;

    /// Get tombstone count for a collection
    async fn tombstone_count(&self, collection: &str) -> Result<i64>;

    /// Clear all records for a collection
    async fn clear_collection(&self, collection: &str) -> Result<u64>;

    /// Compute hash for a local file (helper method).
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read.
    fn compute_hash(&self, path: &Path) -> Result<String>;
}
