use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;

use crate::error::Result;
use crate::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};

/// Vector Store Administrative Operations
///
/// Defines administrative and monitoring operations for vector stores.
/// This trait is separated to keep trait sizes manageable per SOLID principles.
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::providers::vector_store::VectorStoreAdmin;
/// use mcb_domain::value_objects::CollectionId;
/// use std::sync::Arc;
///
/// async fn check_collection(provider: Arc<dyn VectorStoreAdmin>) -> mcb_domain::Result<()> {
///     let id = CollectionId::new("code_embeddings");
///     // Check if a collection exists
///     if provider.collection_exists(&id).await? {
///         let stats = provider.get_stats(&id).await?;
///         println!("Collection stats: {:?}", stats);
///
///         // Flush pending writes
///         provider.flush(&id).await?;
///     }
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait VectorStoreAdmin: Send + Sync {
    /// Check if a collection exists
    ///
    /// # Arguments
    /// * `collection` - ID of the collection to check
    ///
    /// # Returns
    /// Ok(true) if collection exists, Ok(false) if it doesn't exist, Error if check failed
    async fn collection_exists(&self, collection: &CollectionId) -> Result<bool>;

    /// Get statistics about a collection
    ///
    /// # Arguments
    /// * `collection` - ID of the collection to get stats for
    ///
    /// # Returns
    /// Ok(hashmap) containing various statistics about the collection
    async fn get_stats(&self, collection: &CollectionId) -> Result<HashMap<String, Value>>;

    /// Flush pending operations for a collection
    ///
    /// # Arguments
    /// * `collection` - ID of the collection to flush
    ///
    /// # Returns
    /// Ok(()) if flush completed successfully, Error if flush failed
    async fn flush(&self, collection: &CollectionId) -> Result<()>;

    /// Get the name/identifier of this vector store provider
    ///
    /// # Returns
    /// A string identifier for the provider (e.g., "milvus", "edgevec", "null")
    fn provider_name(&self) -> &str;

    /// Health check for the provider (default implementation)
    async fn health_check(&self) -> Result<()> {
        // Default implementation - try a simple operation
        self.collection_exists(
            &"00000000-0000-0000-0000-000000000000"
                .parse::<CollectionId>()
                .expect("health check UUID must be valid"),
        )
        .await?;
        Ok(())
    }
}

/// Enterprise Vector Storage Interface
///
/// Defines the business contract for vector storage systems that persist and
/// retrieve semantic embeddings at enterprise scale. This abstraction supports
/// multiple storage backends from in-memory development stores to production
/// Milvus clusters, ensuring optimal performance for different business needs.
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::providers::vector_store::VectorStoreProvider;
/// use mcb_domain::value_objects::CollectionId;
/// use std::sync::Arc;
///
/// async fn index_code(provider: Arc<dyn VectorStoreProvider>) -> mcb_domain::Result<()> {
///     let id = CollectionId::new("rust_code");
///     // Create a collection for code embeddings
///     provider.create_collection(&id, 384).await?;
///
///     // Search for similar code
///     let query_vec = vec![0.1f32; 384];
///     let results = provider.search_similar(&id, &query_vec, 10, None).await?;
///     for result in results {
///         println!("Found: {} (score: {})", result.file_path, result.score);
///     }
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait VectorStoreProvider: VectorStoreAdmin + VectorStoreBrowser + Send + Sync {
    /// Create a new vector collection with specified dimensions
    ///
    /// # Arguments
    /// * `collection` - ID of the collection to create
    /// * `dimensions` - Number of dimensions for vectors in this collection
    ///
    /// # Returns
    /// Ok(()) if collection was created successfully, Error if creation failed
    async fn create_collection(&self, collection: &CollectionId, dimensions: usize) -> Result<()>;

    /// Delete an existing vector collection
    ///
    /// # Arguments
    /// * `collection` - ID of the collection to delete
    ///
    /// # Returns
    /// Ok(()) if collection was deleted successfully, Error if deletion failed
    async fn delete_collection(&self, collection: &CollectionId) -> Result<()>;

    /// Insert vectors into a collection with associated metadata
    ///
    /// # Arguments
    /// * `collection` - ID of the collection to insert into
    /// * `vectors` - Slice of embedding vectors to insert
    /// * `metadata` - Vector of metadata maps, one per vector
    ///
    /// # Returns
    /// Ok(vector_of_ids) containing the IDs assigned to each inserted vector
    async fn insert_vectors(
        &self,
        collection: &CollectionId,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>>;

    /// Search for vectors similar to a query vector
    ///
    /// # Arguments
    /// * `collection` - ID of the collection to search in
    /// * `query_vector` - The query vector to find similar vectors for
    /// * `limit` - Maximum number of results to return
    /// * `filter` - Optional filter expression to restrict search scope
    ///
    /// # Returns
    /// Ok(vector_of_results) containing the search results ordered by similarity
    async fn search_similar(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>>;

    /// Delete vectors by their IDs
    ///
    /// # Arguments
    /// * `collection` - ID of the collection to delete from
    /// * `ids` - Slice of vector IDs to delete
    ///
    /// # Returns
    /// Ok(()) if all vectors were deleted successfully, Error if deletion failed
    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()>;

    /// Retrieve vectors by their IDs
    ///
    /// # Arguments
    /// * `collection` - ID of the collection to retrieve from
    /// * `ids` - Slice of vector IDs to retrieve
    ///
    /// # Returns
    /// Ok(vector_of_results) containing the requested vectors with their metadata
    async fn get_vectors_by_ids(
        &self,
        collection: &CollectionId,
        ids: &[String],
    ) -> Result<Vec<SearchResult>>;

    /// List vectors in a collection with pagination
    ///
    /// # Arguments
    /// * `collection` - ID of the collection to list vectors from
    /// * `limit` - Maximum number of vectors to return
    ///
    /// # Returns
    /// Ok(vector_of_results) containing the vectors in the collection
    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
}

/// Vector Store Browse Operations for Admin UI
///
/// Provides collection and file browsing capabilities for the Admin UI.
/// This trait extends the base vector store functionality with navigation
/// operations useful for exploring indexed codebases.
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::providers::vector_store::VectorStoreBrowser;
/// use mcb_domain::value_objects::CollectionId;
/// use std::sync::Arc;
///
/// async fn browse_collections(provider: Arc<dyn VectorStoreBrowser>) -> mcb_domain::Result<()> {
///     // List all indexed collections
///     let collections = provider.list_collections().await?;
///     for coll in collections {
///         println!("Collection: {} ({} vectors)", coll.id, coll.vector_count);
///     }
///
///     // List files in a collection
///     let id = CollectionId::new("my-project");
///     let files = provider.list_file_paths(&id, 100).await?;
///     for file in files {
///         println!("File: {}", file.path);
///     }
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait VectorStoreBrowser: Send + Sync {
    /// List all collections with their statistics
    ///
    /// Returns metadata about all indexed collections, including
    /// vector counts, file counts, and provider information.
    ///
    /// # Returns
    /// Ok(vector_of_collection_info) containing info about all collections
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>>;

    /// List unique file paths in a collection
    ///
    /// Returns information about all files indexed in a collection,
    /// useful for building file browser UIs.
    ///
    /// # Arguments
    /// * `collection` - ID of the collection to list files from
    /// * `limit` - Maximum number of files to return
    ///
    /// # Returns
    /// Ok(vector_of_file_info) containing info about indexed files
    async fn list_file_paths(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<FileInfo>>;

    /// Get all chunks for a specific file path
    ///
    /// Retrieves all code chunks that were extracted from a specific
    /// file, ordered by line number.
    ///
    /// # Arguments
    /// * `collection` - ID of the collection to search in
    /// * `file_path` - Path of the file to get chunks for
    ///
    /// # Returns
    /// Ok(vector_of_results) containing chunks from the specified file
    async fn get_chunks_by_file(
        &self,
        collection: &CollectionId,
        file_path: &str,
    ) -> Result<Vec<SearchResult>>;
}
