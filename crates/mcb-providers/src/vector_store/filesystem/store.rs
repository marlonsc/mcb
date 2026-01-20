//! Filesystem vector store implementation
//!
//! Core struct and internal methods for the filesystem-based vector store.

use super::config::FilesystemVectorStoreConfig;
use super::file_utils;
use super::types::{IndexEntry, ShardMetadata};
use crate::constants::FILESYSTEM_BYTES_PER_DIMENSION;
use crate::utils::JsonExt;
use dashmap::DashMap;
use mcb_domain::error::{Error, Result};
use mcb_domain::value_objects::SearchResult;
use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// Filesystem vector store implementation
#[derive(Clone)]
pub struct FilesystemVectorStore {
    pub(super) config: FilesystemVectorStoreConfig,
    /// Global index cache ((collection, ID) -> IndexEntry)
    pub(super) index_cache: Arc<DashMap<(String, String), IndexEntry>>,
    /// Shard metadata cache ((collection, shard_id) -> ShardMetadata)
    pub(super) shard_cache: Arc<DashMap<(String, u32), ShardMetadata>>,
    /// Next shard ID to use per collection
    pub(super) next_shard_ids: Arc<DashMap<String, Arc<AtomicU32>>>,
}

// =============================================================================
// Constructor
// =============================================================================

impl FilesystemVectorStore {
    /// Create a new filesystem vector store
    pub async fn new(config: FilesystemVectorStoreConfig) -> Result<Self> {
        // Ensure base directory exists
        tokio::fs::create_dir_all(&config.base_path)
            .await
            .map_err(|e| Error::io(format!("Failed to create base directory: {}", e)))?;

        let store = Self {
            config,
            index_cache: Arc::new(DashMap::new()),
            shard_cache: Arc::new(DashMap::new()),
            next_shard_ids: Arc::new(DashMap::new()),
        };

        Ok(store)
    }
}

// =============================================================================
// State Management - Load and save collection state to disk
// =============================================================================

impl FilesystemVectorStore {
    /// Load existing state from disk for a collection
    pub(super) async fn load_collection_state(&self, collection: &str) -> Result<()> {
        // Load global index
        let index_path = self
            .config
            .base_path
            .join(format!("{}_index.json", collection));
        if file_utils::exists(&index_path).await {
            let index: HashMap<String, IndexEntry> =
                file_utils::read_json(&index_path, "collection index").await?;
            for (id, entry) in index {
                self.index_cache.insert((collection.to_string(), id), entry);
            }
        }

        // Load shard metadata
        let shards_path = self.config.base_path.join(format!("{}_shards", collection));
        if shards_path.exists() {
            let mut entries = tokio::fs::read_dir(&shards_path)
                .await
                .map_err(|e| Error::io(format!("Failed to read shards directory: {}", e)))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| Error::io(format!("Failed to read directory entry: {}", e)))?
            {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("meta") {
                    let metadata: ShardMetadata =
                        file_utils::read_json(&path, "shard metadata").await?;
                    self.shard_cache
                        .insert((collection.to_string(), metadata.shard_id), metadata);
                }
            }
        }

        // Find next shard ID
        let max_shard_id = self
            .shard_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .map(|r| r.value().shard_id)
            .max()
            .unwrap_or(0);

        self.next_shard_ids.insert(
            collection.to_string(),
            Arc::new(AtomicU32::new(max_shard_id + 1)),
        );

        Ok(())
    }

    /// Save state to disk for a collection
    pub(super) async fn save_collection_state(&self, collection: &str) -> Result<()> {
        // Save global index
        let index_path = self
            .config
            .base_path
            .join(format!("{}_index.json", collection));
        let index: HashMap<String, IndexEntry> = self
            .index_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .map(|r| (r.key().1.clone(), r.value().clone()))
            .collect();
        file_utils::write_json(&index_path, &index, "collection index").await?;

        // Save shard metadata
        let shards_path = self.config.base_path.join(format!("{}_shards", collection));
        for r in self.shard_cache.iter() {
            let (c, shard_id) = r.key();
            if c == collection {
                let meta_path = shards_path.join(format!("shard_{}.meta", shard_id));
                file_utils::ensure_dir_write_json(&meta_path, r.value(), "shard metadata").await?;
            }
        }
        Ok(())
    }
}

// =============================================================================
// Shard Management - Path utilities, allocation, and capacity
// =============================================================================

impl FilesystemVectorStore {
    /// Get shard file path for a collection
    pub(super) fn get_shard_path(&self, collection: &str, shard_id: u32) -> PathBuf {
        self.config
            .base_path
            .join(format!("{}_shards", collection))
            .join(format!("shard_{}.dat", shard_id))
    }

    /// Allocate next shard ID
    pub(super) fn allocate_shard_id(&self, collection: &str) -> u32 {
        let entry = self
            .next_shard_ids
            .entry(collection.to_string())
            .or_insert_with(|| Arc::new(AtomicU32::new(0)));
        entry.value().fetch_add(1, Ordering::SeqCst)
    }

    /// Create new shard if needed
    pub(super) async fn ensure_shard_capacity(
        &self,
        collection: &str,
        shard_id: u32,
    ) -> Result<()> {
        let shard_path = self.get_shard_path(collection, shard_id);

        if !file_utils::exists(&shard_path).await {
            // Create shard file with empty content
            file_utils::ensure_dir_write(&shard_path, &[], "shard file").await?;

            let metadata = ShardMetadata {
                shard_id,
                vector_count: 0,
                vectors_offset: 0,
                vectors_size: 0,
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            self.shard_cache
                .insert((collection.to_string(), shard_id), metadata);
        }
        Ok(())
    }

    /// Find optimal shard for new vector
    pub(super) fn find_optimal_shard(&self, collection: &str) -> u32 {
        // Find shard with most available capacity
        let mut best_shard = None;
        let mut min_vectors = usize::MAX;

        for r in self.shard_cache.iter() {
            let (c, shard_id) = r.key();
            if c == collection {
                let metadata = r.value();
                if metadata.vector_count < self.config.max_vectors_per_shard
                    && metadata.vector_count < min_vectors
                {
                    min_vectors = metadata.vector_count;
                    best_shard = Some(*shard_id);
                }
            }
        }

        if let Some(shard_id) = best_shard {
            shard_id
        } else {
            // Allocate new shard
            self.allocate_shard_id(collection)
        }
    }
}

// =============================================================================
// Vector I/O - Read and write vectors to shard files
// =============================================================================

impl FilesystemVectorStore {
    /// Write vector to shard
    pub(super) async fn write_vector_to_shard(
        &self,
        collection: &str,
        shard_id: u32,
        _id: &str,
        vector: &[f32],
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<u64> {
        self.ensure_shard_capacity(collection, shard_id).await?;

        let shard_path = self.get_shard_path(collection, shard_id);

        let vector_bytes = self.vector_to_bytes(vector);
        let metadata_bytes = serde_json::to_vec(metadata)
            .map_err(|e| Error::internal(format!("Failed to serialize metadata: {}", e)))?;
        let metadata_len = metadata_bytes.len() as u32;

        let (offset, total_shard_size) = tokio::task::spawn_blocking(move || {
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&shard_path)?;

            let offset = file.metadata()?.len();
            file.seek(std::io::SeekFrom::End(0))?;
            file.write_all(&vector_bytes)?;
            file.write_all(&metadata_len.to_le_bytes())?;
            file.write_all(&metadata_bytes)?;

            let total_size = file.metadata()?.len();
            Ok::<_, std::io::Error>((offset, total_size))
        })
        .await
        .map_err(|e| Error::internal(format!("Blocking task failed: {}", e)))?
        .map_err(|e| Error::io(format!("Failed to write to shard: {}", e)))?;

        // Update shard metadata
        if let Some(mut shard_meta) = self
            .shard_cache
            .get_mut(&(collection.to_string(), shard_id))
        {
            shard_meta.vector_count += 1;
            shard_meta.vectors_size = total_shard_size;
        }

        Ok(offset)
    }

    /// Read vector from shard
    pub(super) async fn read_vector_from_shard(
        &self,
        collection: &str,
        shard_id: u32,
        offset: u64,
    ) -> Result<(Vec<f32>, HashMap<String, serde_json::Value>)> {
        let shard_path = self.get_shard_path(collection, shard_id);
        let dimensions = self.config.dimensions;

        tokio::task::spawn_blocking(move || {
            let mut file = std::fs::File::open(&shard_path)?;
            file.seek(std::io::SeekFrom::Start(offset))?;

            // Read vector data
            let mut bytes = vec![0u8; dimensions * FILESYSTEM_BYTES_PER_DIMENSION];
            file.read_exact(&mut bytes)?;
            let mut vector = Vec::with_capacity(dimensions);
            for chunk in bytes.chunks_exact(FILESYSTEM_BYTES_PER_DIMENSION) {
                let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                vector.push(value);
            }

            // Read metadata length
            let mut metadata_len_bytes = [0u8; 4];
            file.read_exact(&mut metadata_len_bytes)?;
            let metadata_len = u32::from_le_bytes(metadata_len_bytes);

            // Read metadata
            let mut metadata_bytes = vec![0u8; metadata_len as usize];
            file.read_exact(&mut metadata_bytes)?;
            let metadata: HashMap<String, serde_json::Value> =
                serde_json::from_slice(&metadata_bytes)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            Ok((vector, metadata))
        })
        .await
        .map_err(|e| Error::internal(format!("Blocking task failed: {}", e)))?
        .map_err(|e: std::io::Error| Error::io(format!("Failed to read from shard: {}", e)))
    }
}

// =============================================================================
// Search - Similarity search implementations
// =============================================================================

impl FilesystemVectorStore {
    /// Perform similarity search using brute force
    pub(super) async fn brute_force_search(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Collect index entries first to avoid holding DashMap iterator across await points
        let entries: Vec<IndexEntry> = self
            .index_cache
            .iter()
            .filter(|r| r.key().0 == collection)
            .map(|r| r.value().clone())
            .collect();

        for entry in entries {
            if let Ok((vector, metadata)) = self
                .read_vector_from_shard(collection, entry.shard_id, entry.offset)
                .await
            {
                let similarity = self.cosine_similarity(query_vector, &vector);

                let file_path = metadata.string_or("file_path", "unknown");
                let start_line = metadata
                    .opt_u64("start_line")
                    .or_else(|| metadata.opt_u64("line_number"))
                    .unwrap_or(0) as u32;
                let content = metadata.string_or("content", "");
                let language = metadata.string_or("language", "unknown");

                results.push(SearchResult {
                    id: entry.id.clone(),
                    file_path,
                    start_line,
                    content,
                    score: similarity as f64,
                    language,
                });
            }
        }

        // Sort by similarity (descending) and take top results
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }
}

// =============================================================================
// Math Utilities - Vector operations and conversions
// =============================================================================

impl FilesystemVectorStore {
    /// Calculate cosine similarity between two vectors
    pub(super) fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let (dot_product, norm_a, norm_b) = a
            .iter()
            .zip(b.iter())
            .fold((0.0, 0.0, 0.0), |(dot, na, nb), (&x, &y)| {
                (dot + x * y, na + x * x, nb + y * y)
            });

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a.sqrt() * norm_b.sqrt())
        }
    }

    /// Convert vector to bytes
    pub(super) fn vector_to_bytes(&self, vector: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(vector.len() * FILESYSTEM_BYTES_PER_DIMENSION);
        for &value in vector {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}
