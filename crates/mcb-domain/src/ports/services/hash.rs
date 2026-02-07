use std::path::Path;

use async_trait::async_trait;

use crate::error::Result;

/// File hash state management port
#[async_trait]
pub trait FileHashService: Send + Sync {
    /// Check whether the file hash differs from what is stored or if it's new
    async fn has_changed(
        &self,
        collection: &str,
        file_path: &str,
        current_hash: &str,
    ) -> Result<bool>;

    /// Insert or update the stored hash for a file
    async fn upsert_hash(&self, collection: &str, file_path: &str, hash: &str) -> Result<()>;

    /// List all files currently tracked in a collection
    async fn get_indexed_files(&self, collection: &str) -> Result<Vec<String>>;

    /// Mark a file as deleted so it will be re-indexed later
    async fn mark_deleted(&self, collection: &str, file_path: &str) -> Result<()>;

    /// Compute the hash value for a file path
    fn compute_hash(path: &Path) -> Result<String>;
}
