//! Vector store provider ports.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::error::Result;
use crate::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};

/// Administrative operations for vector database collections.
#[async_trait]
pub trait VectorStoreAdmin: Send + Sync {
    /// Check if the specified collection exists in the vector store.
    async fn collection_exists(&self, collection: &CollectionId) -> Result<bool>;

    /// Retrieve performance and storage statistics for a collection.
    async fn get_stats(
        &self,
        collection: &CollectionId,
    ) -> Result<HashMap<String, serde_json::Value>>;

    /// Ensure all pending writes are committed and searchable.
    async fn flush(&self, collection: &CollectionId) -> Result<()>;

    /// Get the unique name of this vector store implementation.
    fn provider_name(&self) -> &str;

    /// Perform a basic health check on the connection.
    async fn health_check(&self) -> Result<()> {
        let health_check_id = CollectionId::from_name("__health_check__");
        self.collection_exists(&health_check_id).await?;
        Ok(())
    }
}

/// Read-only discovery and browsing of the vector database.
#[async_trait]
pub trait VectorStoreBrowser: Send + Sync {
    /// List all collections available in the vector store.
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>>;

    /// List unique file paths present in a collection (paginated).
    async fn list_file_paths(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<FileInfo>>;

    /// Retrieve all stored code chunks for a specific file.
    async fn get_chunks_by_file(
        &self,
        collection: &CollectionId,
        file_path: &str,
    ) -> Result<Vec<SearchResult>>;
}

/// Unified interface for vector store operations.
#[async_trait]
pub trait VectorStoreProvider: VectorStoreAdmin + VectorStoreBrowser + Send + Sync {
    /// Create a new collection with specified embedding dimensions.
    async fn create_collection(&self, collection: &CollectionId, dimensions: usize) -> Result<()>;

    /// Permanently delete a collection and all its vectors.
    async fn delete_collection(&self, collection: &CollectionId) -> Result<()>;

    /// Insert a batch of vectors with associated metadata.
    async fn insert_vectors(
        &self,
        collection: &CollectionId,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>>;

    /// Find vectors similar to the provided query vector.
    async fn search_similar(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>>;

    /// Delete specific vectors by their unique IDs.
    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()>;

    /// Retrieve specific search results by their vector record IDs.
    async fn get_vectors_by_ids(
        &self,
        collection: &CollectionId,
        ids: &[String],
    ) -> Result<Vec<SearchResult>>;

    /// List vectors in a collection (paginated).
    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
}
