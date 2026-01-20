//! Filesystem-optimized vector store implementation
//!
//! Provides high-performance vector storage using memory-mapped files
//! with optimized indexing for production workloads.
//!
//! ## Module Structure
//!
//! - `config` - Configuration types
//! - `types` - Internal shard and index types
//! - `file_utils` - Async file operations
//! - `store` - Core implementation

mod config;
mod file_utils;
mod store;
mod types;

pub use config::FilesystemVectorStoreConfig;
pub use store::FilesystemVectorStore;

use crate::utils::JsonExt;
use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionInfo, Embedding, FileInfo, SearchResult};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use types::IndexEntry;

// =============================================================================
// VectorStoreAdmin Implementation
// =============================================================================

#[async_trait]
impl VectorStoreAdmin for FilesystemVectorStore {
    async fn collection_exists(&self, name: &str) -> Result<bool> {
        let index_path = self.config.base_path.join(format!("{}_index.json", name));
        Ok(index_path.exists())
    }

    async fn get_stats(&self, collection: &str) -> Result<HashMap<String, serde_json::Value>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        let mut stats = HashMap::new();
        stats.insert("collection".to_string(), serde_json::json!(collection));

        let total_vectors = self
            .index_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .count();
        stats.insert(
            "total_vectors".to_string(),
            serde_json::json!(total_vectors),
        );

        let total_shards = self
            .shard_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .count();
        stats.insert("total_shards".to_string(), serde_json::json!(total_shards));

        stats.insert(
            "dimensions".to_string(),
            serde_json::json!(self.config.dimensions),
        );

        let total_size: u64 = self
            .shard_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .map(|r| r.value().vectors_size)
            .sum();

        stats.insert(
            "total_size_bytes".to_string(),
            serde_json::json!(total_size),
        );

        Ok(stats)
    }

    async fn flush(&self, collection: &str) -> Result<()> {
        self.save_collection_state(collection).await
    }

    fn provider_name(&self) -> &str {
        "filesystem"
    }
}

// =============================================================================
// VectorStoreProvider Implementation
// =============================================================================

#[async_trait]
impl VectorStoreProvider for FilesystemVectorStore {
    async fn create_collection(&self, name: &str, _dimensions: usize) -> Result<()> {
        // Try to load existing collection, if it doesn't exist, create it
        if !self.collection_exists(name).await? {
            // Collection doesn't exist, save initial empty state
            self.save_collection_state(name).await?;
        } else {
            self.load_collection_state(name).await?;
        }

        Ok(())
    }

    async fn delete_collection(&self, name: &str) -> Result<()> {
        // Remove all files for this collection
        let collection_path = self.config.base_path.join(format!("{}_shards", name));
        if collection_path.exists() {
            tokio::fs::remove_dir_all(&collection_path)
                .await
                .map_err(|e| Error::io(format!("Failed to delete collection shards: {}", e)))?;
        }

        let index_path = self.config.base_path.join(format!("{}_index.json", name));
        if index_path.exists() {
            tokio::fs::remove_file(index_path)
                .await
                .map_err(|e| Error::io(format!("Failed to delete collection index: {}", e)))?;
        }

        // Clear caches
        self.index_cache.retain(|k, _| k.0 != name);
        self.shard_cache.retain(|k, _| k.0 != name);
        self.next_shard_ids.remove(name);

        Ok(())
    }

    async fn insert_vectors(
        &self,
        collection: &str,
        vectors: &[Embedding],
        metadata: Vec<std::collections::HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        let mut ids = Vec::new();

        for (i, (vector, meta)) in vectors.iter().zip(metadata.iter()).enumerate() {
            let id = format!(
                "{}_{}_{}",
                collection,
                i,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            let shard_id = self.find_optimal_shard(collection);
            let offset = self
                .write_vector_to_shard(collection, shard_id, &id, &vector.vector, meta)
                .await?;

            let index_entry = IndexEntry {
                id: id.clone(),
                shard_id,
                offset,
                metadata: meta.clone(),
            };

            self.index_cache
                .insert((collection.to_string(), id.clone()), index_entry);
            ids.push(id);
        }

        // Save state
        self.save_collection_state(collection).await?;

        Ok(ids)
    }

    async fn search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        self.brute_force_search(collection, query_vector, limit)
            .await
    }

    async fn delete_vectors(&self, collection: &str, ids: &[String]) -> Result<()> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        // Remove from index
        for id in ids {
            self.index_cache
                .remove(&(collection.to_string(), id.clone()));
        }

        // Save state
        self.save_collection_state(collection).await?;
        Ok(())
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &str,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        let mut results = Vec::new();
        for id in ids {
            if let Some(entry) = self.index_cache.get(&(collection.to_string(), id.clone())) {
                if let Ok((_, metadata)) = self
                    .read_vector_from_shard(collection, entry.shard_id, entry.offset)
                    .await
                {
                    let file_path = metadata.string_or("file_path", "unknown");
                    let start_line = metadata
                        .opt_u64("start_line")
                        .or_else(|| metadata.opt_u64("line_number"))
                        .unwrap_or(0) as u32;
                    let content = metadata.string_or("content", "");
                    let language = metadata.string_or("language", "unknown");

                    results.push(SearchResult {
                        id: id.clone(),
                        file_path,
                        start_line,
                        content,
                        score: 1.0,
                        language,
                    });
                }
            }
        }
        Ok(results)
    }

    async fn list_vectors(&self, collection: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        let mut results = Vec::new();
        let entries: Vec<_> = self
            .index_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .take(limit)
            .map(|r| (r.key().1.clone(), r.value().clone()))
            .collect();

        for (id, entry) in entries {
            if let Ok((_vector, metadata)) = self
                .read_vector_from_shard(collection, entry.shard_id, entry.offset)
                .await
            {
                let file_path = metadata.string_or("file_path", "unknown");
                let start_line = metadata
                    .opt_u64("start_line")
                    .or_else(|| metadata.opt_u64("line_number"))
                    .unwrap_or(0) as u32;
                let content = metadata.string_or("content", "");
                let language = metadata.string_or("language", "unknown");

                results.push(SearchResult {
                    id,
                    file_path,
                    start_line,
                    content,
                    score: 1.0,
                    language,
                });
            }
        }
        Ok(results)
    }
}

// =============================================================================
// VectorStoreBrowser Implementation
// =============================================================================

#[async_trait]
impl VectorStoreBrowser for FilesystemVectorStore {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        // Find all collection index files
        let mut collections = Vec::new();

        let entries = tokio::fs::read_dir(&self.config.base_path)
            .await
            .map_err(|e| Error::io(format!("Failed to read base directory: {}", e)))?;

        let mut entries = entries;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| Error::io(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with("_index.json") {
                    let collection_name = name.trim_end_matches("_index.json");

                    // Get stats for this collection
                    let stats = self.get_stats(collection_name).await.unwrap_or_default();
                    let vector_count = stats
                        .get("total_vectors")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    // Count unique files from index cache
                    let file_paths: HashSet<String> = self
                        .index_cache
                        .iter()
                        .filter(|r| r.key().0 == collection_name)
                        .filter_map(|r| {
                            r.value()
                                .metadata
                                .get("file_path")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect();
                    let file_count = file_paths.len() as u64;

                    collections.push(CollectionInfo::new(
                        collection_name,
                        vector_count,
                        file_count,
                        None,
                        self.provider_name(),
                    ));
                }
            }
        }

        Ok(collections)
    }

    async fn list_file_paths(&self, collection: &str, limit: usize) -> Result<Vec<FileInfo>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        // Aggregate file info from index cache
        let mut file_map: HashMap<String, (u32, String)> = HashMap::new();

        for entry in self.index_cache.iter() {
            if entry.key().0 == collection {
                if let Some(file_path) = entry
                    .value()
                    .metadata
                    .get("file_path")
                    .and_then(|v| v.as_str())
                {
                    let language = entry
                        .value()
                        .metadata
                        .get("language")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let e = file_map
                        .entry(file_path.to_string())
                        .or_insert((0, language));
                    e.0 += 1; // Increment chunk count
                }
            }
        }

        let files: Vec<FileInfo> = file_map
            .into_iter()
            .take(limit)
            .map(|(path, (chunk_count, language))| FileInfo::new(path, chunk_count, language, None))
            .collect();

        Ok(files)
    }

    async fn get_chunks_by_file(
        &self,
        collection: &str,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        // Ensure state is loaded
        if !self.next_shard_ids.contains_key(collection) {
            self.load_collection_state(collection).await?;
        }

        let mut results = Vec::new();

        // Find all entries for this file
        let entries: Vec<_> = self
            .index_cache
            .iter()
            .filter(|r| {
                r.key().0 == collection
                    && r.value()
                        .metadata
                        .get("file_path")
                        .and_then(|v| v.as_str())
                        .is_some_and(|p| p == file_path)
            })
            .map(|r| (r.key().1.clone(), r.value().clone()))
            .collect();

        for (id, entry) in entries {
            if let Ok((_, metadata)) = self
                .read_vector_from_shard(collection, entry.shard_id, entry.offset)
                .await
            {
                let start_line = metadata
                    .opt_u64("start_line")
                    .or_else(|| metadata.opt_u64("line_number"))
                    .unwrap_or(0) as u32;
                let content = metadata.string_or("content", "");
                let language = metadata.string_or("language", "unknown");

                results.push(SearchResult {
                    id,
                    file_path: file_path.to_string(),
                    start_line,
                    content,
                    score: 1.0,
                    language,
                });
            }
        }

        // Sort by start_line
        results.sort_by_key(|r| r.start_line);

        Ok(results)
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use mcb_application::ports::registry::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
};

/// Factory function for creating filesystem vector store provider instances.
fn filesystem_factory(
    config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    let base_path = config.uri.clone().ok_or_else(|| {
        "Filesystem store requires 'uri' configuration (path to storage directory)".to_string()
    })?;
    let dimensions = config.dimensions.ok_or_else(|| {
        "Filesystem store requires 'dimensions' configuration (embedding vector size)".to_string()
    })?;

    let fs_config = FilesystemVectorStoreConfig {
        base_path: std::path::PathBuf::from(base_path),
        dimensions,
        ..Default::default()
    };

    // Create store synchronously using block_in_place for the async constructor
    let store = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { FilesystemVectorStore::new(fs_config).await })
    })
    .map_err(|e| format!("Failed to create filesystem store: {e}"))?;

    Ok(Arc::new(store))
}

#[linkme::distributed_slice(VECTOR_STORE_PROVIDERS)]
static FILESYSTEM_PROVIDER: VectorStoreProviderEntry = VectorStoreProviderEntry {
    name: "filesystem",
    description: "Filesystem-based vector store (persistent, sharded)",
    factory: filesystem_factory,
};
