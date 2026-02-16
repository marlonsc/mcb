//! `EdgeVec` Vector Store Provider
//!
//! High-performance embedded vector database implementation using `EdgeVec`.
//! `EdgeVec` provides sub-millisecond vector similarity search with HNSW algorithm.
//! This implementation uses the Actor pattern to eliminate locks and ensure non-blocking operation.

use std::collections::HashMap;

use async_trait::async_trait;
use dashmap::DashMap;
use edgevec::hnsw::VectorId;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::providers::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::utils::id;
use mcb_domain::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};
use tokio::sync::{mpsc, oneshot};

use crate::constants::{
    EDGEVEC_DEFAULT_DIMENSIONS, EDGEVEC_HNSW_EF_CONSTRUCTION, EDGEVEC_HNSW_EF_SEARCH,
    EDGEVEC_HNSW_M, EDGEVEC_HNSW_M0, EDGEVEC_QUANTIZATION_TYPE, STATS_FIELD_COLLECTION,
    STATS_FIELD_VECTORS_COUNT,
};

/// `EdgeVec` vector store configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct EdgeVecConfig {
    /// Vector dimensionality
    #[serde(default = "default_dimensions")]
    pub dimensions: usize,

    /// HNSW parameters for index optimization
    #[serde(default)]
    pub hnsw_config: HnswConfig,

    /// Distance metric to use
    #[serde(default)]
    pub metric: MetricType,

    /// Whether to use quantization for memory optimization
    #[serde(default)]
    pub use_quantization: bool,

    /// Quantization configuration
    #[serde(default)]
    pub quantizer_config: QuantizerConfig,
}

fn default_dimensions() -> usize {
    EDGEVEC_DEFAULT_DIMENSIONS
}

/// HNSW configuration for `EdgeVec`
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct HnswConfig {
    /// Maximum connections per node in layers > 0
    #[serde(default = "default_m")]
    pub m: u32,

    /// Maximum connections per node in layer 0
    #[serde(default = "default_m0")]
    pub m0: u32,

    /// Construction-time candidate list size
    #[serde(default = "default_ef_construction")]
    pub ef_construction: u32,

    /// Search-time candidate list size
    #[serde(default = "default_ef_search")]
    pub ef_search: u32,
}

fn default_m() -> u32 {
    EDGEVEC_HNSW_M
}
fn default_m0() -> u32 {
    EDGEVEC_HNSW_M0
}
fn default_ef_construction() -> u32 {
    EDGEVEC_HNSW_EF_CONSTRUCTION
}
fn default_ef_search() -> u32 {
    EDGEVEC_HNSW_EF_SEARCH
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: default_m(),
            m0: default_m0(),
            ef_construction: default_ef_construction(),
            ef_search: default_ef_search(),
        }
    }
}

/// Distance metrics supported by `EdgeVec`
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema, Default)]
pub enum MetricType {
    /// L2 Squared (Euclidean) distance
    L2Squared,
    /// Cosine similarity
    #[default]
    Cosine,
    /// Dot product
    DotProduct,
}

/// Quantization configuration for memory optimization
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct QuantizerConfig {
    /// `EdgeVec` quantization type for scalar quantization.
    #[serde(default)]
    pub quantization_type: String,
}

impl Default for QuantizerConfig {
    fn default() -> Self {
        Self {
            quantization_type: EDGEVEC_QUANTIZATION_TYPE.to_owned(),
        }
    }
}

impl Default for EdgeVecConfig {
    fn default() -> Self {
        Self {
            dimensions: default_dimensions(),
            hnsw_config: HnswConfig::default(),
            metric: MetricType::default(),
            use_quantization: false,
            quantizer_config: QuantizerConfig::default(),
        }
    }
}

/// Core collection management messages
enum CoreMessage {
    CreateCollection {
        name: String,
        tx: oneshot::Sender<Result<()>>,
    },
    DeleteCollection {
        name: String,
        tx: oneshot::Sender<Result<()>>,
    },
    InsertVectors {
        collection: String,
        vectors: Vec<Embedding>,
        metadata: Vec<HashMap<String, serde_json::Value>>,
        tx: oneshot::Sender<Result<Vec<String>>>,
    },
    SearchSimilar {
        collection: String,
        query_vector: Vec<f32>,
        limit: usize,
        tx: oneshot::Sender<Result<Vec<SearchResult>>>,
    },
    DeleteVectors {
        collection: String,
        ids: Vec<String>,
        tx: oneshot::Sender<Result<()>>,
    },
}

/// Query and stats messages
enum QueryMessage {
    GetStats {
        collection: String,
        tx: oneshot::Sender<Result<HashMap<String, serde_json::Value>>>,
    },
    ListVectors {
        collection: String,
        limit: usize,
        tx: oneshot::Sender<Result<Vec<SearchResult>>>,
    },
    GetVectorsByIds {
        collection: String,
        ids: Vec<String>,
        tx: oneshot::Sender<Result<Vec<SearchResult>>>,
    },
    CollectionExists {
        name: String,
        tx: oneshot::Sender<Result<bool>>,
    },
}

/// Browse API messages
enum BrowseMessage {
    ListCollections {
        tx: oneshot::Sender<Result<Vec<CollectionInfo>>>,
    },
    ListFilePaths {
        collection: String,
        limit: usize,
        tx: oneshot::Sender<Result<Vec<FileInfo>>>,
    },
    GetChunksByFile {
        collection: String,
        file_path: String,
        tx: oneshot::Sender<Result<Vec<SearchResult>>>,
    },
}

/// Messages for the `EdgeVec` actor - categorized for OCP compliance
enum EdgeVecMessage {
    Core(CoreMessage),
    Query(QueryMessage),
    Browse(BrowseMessage),
}

/// `EdgeVec` vector store provider implementation using Actor pattern
pub struct EdgeVecVectorStoreProvider {
    sender: mpsc::Sender<EdgeVecMessage>,
    _collection: CollectionId,
}

impl EdgeVecVectorStoreProvider {
    /// Create a new `EdgeVec` vector store provider
    ///
    /// # Errors
    ///
    /// Returns an error if the `EdgeVec` actor fails to initialize.
    pub fn new(config: &EdgeVecConfig) -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);
        let config_clone = config.clone();

        let actor = EdgeVecActor::new(rx, config_clone)?;
        tokio::spawn(async move {
            actor.run().await;
        });

        let generated_collection = CollectionId::from_name(&format!("edgevec-{}", id::generate()));

        Ok(Self {
            sender: tx,
            _collection: generated_collection,
        })
    }

    /// Create a new `EdgeVec` provider with custom collection
    ///
    /// # Errors
    ///
    /// Returns an error if the `EdgeVec` actor fails to initialize.
    pub fn with_collection(config: &EdgeVecConfig, collection: CollectionId) -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);
        let config_clone = config.clone();

        let actor = EdgeVecActor::new(rx, config_clone)?;
        tokio::spawn(async move {
            actor.run().await;
        });

        Ok(Self {
            sender: tx,
            _collection: collection,
        })
    }
}

#[async_trait]
impl VectorStoreAdmin for EdgeVecVectorStoreProvider {
    async fn collection_exists(&self, collection: &CollectionId) -> Result<bool> {
        send_actor_msg!(
            self,
            Query(QueryMessage::CollectionExists {
                name: collection.to_string()
            })
        )
    }

    async fn get_stats(
        &self,
        collection: &CollectionId,
    ) -> Result<HashMap<String, serde_json::Value>> {
        send_actor_msg!(
            self,
            Query(QueryMessage::GetStats {
                collection: collection.to_string()
            })
        )
    }

    async fn flush(&self, _collection: &CollectionId) -> Result<()> {
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "edgevec"
    }
}

#[async_trait]
impl VectorStoreProvider for EdgeVecVectorStoreProvider {
    async fn create_collection(&self, collection: &CollectionId, _dimensions: usize) -> Result<()> {
        send_actor_msg!(
            self,
            Core(CoreMessage::CreateCollection {
                name: collection.to_string()
            })
        )
    }

    async fn delete_collection(&self, collection: &CollectionId) -> Result<()> {
        send_actor_msg!(
            self,
            Core(CoreMessage::DeleteCollection {
                name: collection.to_string()
            })
        )
    }

    async fn insert_vectors(
        &self,
        collection: &CollectionId,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        send_actor_msg!(
            self,
            Core(CoreMessage::InsertVectors {
                collection: collection.to_string(),
                vectors: vectors.to_vec(),
                metadata: metadata
            })
        )
    }

    async fn search_similar(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        send_actor_msg!(
            self,
            Core(CoreMessage::SearchSimilar {
                collection: collection.to_string(),
                query_vector: query_vector.to_vec(),
                limit: limit
            })
        )
    }

    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()> {
        send_actor_msg!(
            self,
            Core(CoreMessage::DeleteVectors {
                collection: collection.to_string(),
                ids: ids.to_vec()
            })
        )
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &CollectionId,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        send_actor_msg!(
            self,
            Query(QueryMessage::GetVectorsByIds {
                collection: collection.to_string(),
                ids: ids.to_vec()
            })
        )
    }

    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        send_actor_msg!(
            self,
            Query(QueryMessage::ListVectors {
                collection: collection.to_string(),
                limit: limit
            })
        )
    }
}

#[async_trait]
impl VectorStoreBrowser for EdgeVecVectorStoreProvider {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        send_actor_msg!(self, Browse(BrowseMessage::ListCollections {}))
    }

    async fn list_file_paths(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<FileInfo>> {
        send_actor_msg!(
            self,
            Browse(BrowseMessage::ListFilePaths {
                collection: collection.to_string(),
                limit: limit
            })
        )
    }

    async fn get_chunks_by_file(
        &self,
        collection: &CollectionId,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        send_actor_msg!(
            self,
            Browse(BrowseMessage::GetChunksByFile {
                collection: collection.to_string(),
                file_path: file_path.to_owned()
            })
        )
    }
}

struct EdgeVecActor {
    receiver: mpsc::Receiver<EdgeVecMessage>,
    index: edgevec::HnswIndex,
    storage: edgevec::VectorStorage,
    metadata_store: DashMap<String, HashMap<String, serde_json::Value>>,
    id_map: DashMap<String, VectorId>,
    config: EdgeVecConfig,
}

// =============================================================================
// Core - Constructor and HNSW configuration
// =============================================================================

impl EdgeVecActor {
    fn new(receiver: mpsc::Receiver<EdgeVecMessage>, config: EdgeVecConfig) -> Result<Self> {
        let hnsw_config = edgevec::HnswConfig {
            m: config.hnsw_config.m,
            m0: config.hnsw_config.m0,
            ef_construction: config.hnsw_config.ef_construction,
            ef_search: config.hnsw_config.ef_search,
            dimensions: config.dimensions as u32,
            metric: match config.metric {
                MetricType::L2Squared => edgevec::HnswConfig::METRIC_L2_SQUARED,
                MetricType::Cosine => edgevec::HnswConfig::METRIC_COSINE,
                MetricType::DotProduct => edgevec::HnswConfig::METRIC_DOT_PRODUCT,
            },
            _reserved: [0; 2],
        };

        let storage = edgevec::VectorStorage::new(&hnsw_config, None);
        let index = edgevec::HnswIndex::new(hnsw_config, &storage)
            .map_err(|e| Error::internal(format!("Failed to create EdgeVec HNSW index: {e}")))?;

        Ok(Self {
            receiver,
            index,
            storage,
            metadata_store: DashMap::new(),
            id_map: DashMap::new(),
            config,
        })
    }
}

// =============================================================================
// Collection Handlers - Create, delete, exists
// =============================================================================

impl EdgeVecActor {
    fn handle_create_collection(&self, name: String) -> Result<()> {
        self.metadata_store.insert(name, HashMap::new());
        Ok(())
    }

    fn handle_delete_collection(&mut self, name: &str) -> Result<()> {
        if let Some((_, collection_metadata)) = self.metadata_store.remove(name) {
            for external_id in collection_metadata.keys() {
                if let Some(vector_id) = self.id_map.remove(external_id) {
                    let _ = self.index.soft_delete(vector_id.1);
                }
            }
        }
        Ok(())
    }

    fn handle_collection_exists(&self, name: &str) -> Result<bool> {
        Ok(self.metadata_store.contains_key(name))
    }
}

// =============================================================================
// Vector CRUD Handlers - Insert, delete, get, list vectors
// =============================================================================

impl EdgeVecActor {
    fn handle_insert_vectors(
        &mut self,
        collection: &str,
        vectors: Vec<Embedding>,
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        let mut ids = Vec::with_capacity(vectors.len());
        let mut collection_metadata = self
            .metadata_store
            .entry(collection.to_owned())
            .or_default();

        for (vector, meta) in vectors.into_iter().zip(metadata.into_iter()) {
            let external_id = format!("{}_{}", collection, id::generate());

            match self.index.insert(&vector.vector, &mut self.storage) {
                Ok(vector_id) => {
                    self.id_map.insert(external_id.clone(), vector_id);
                    let mut enriched_metadata = meta.clone();
                    enriched_metadata.insert("id".to_owned(), serde_json::json!(external_id));
                    collection_metadata
                        .insert(external_id.clone(), serde_json::json!(enriched_metadata));
                    ids.push(external_id);
                }
                Err(e) => {
                    return Err(Error::internal(format!("Failed to insert vector: {e}")));
                }
            }
        }
        Ok(ids)
    }

    fn handle_delete_vectors(&mut self, collection: &str, ids: Vec<String>) -> Result<()> {
        if let Some(mut collection_metadata) = self.metadata_store.get_mut(collection) {
            for id in ids {
                if let Some((_, vector_id)) = self.id_map.remove(&id) {
                    let _ = self.index.soft_delete(vector_id);
                }
                collection_metadata.remove(&id);
            }
        }
        Ok(())
    }

    fn handle_get_vectors_by_ids(&self, collection: &str, ids: Vec<String>) -> Vec<SearchResult> {
        let mut final_results = Vec::new();
        if let Some(collection_metadata) = self.metadata_store.get(collection) {
            for id in ids {
                if let Some(meta_val) = collection_metadata.get(&id) {
                    let meta = meta_val.as_object().cloned().unwrap_or_default();
                    final_results.push(SearchResult {
                        id: id.clone(),
                        file_path: meta
                            .get("file_path")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown")
                            .to_owned(),
                        start_line: meta
                            .get("start_line")
                            .and_then(serde_json::Value::as_u64)
                            .or_else(|| meta.get("line_number").and_then(serde_json::Value::as_u64))
                            .unwrap_or(0) as u32,
                        content: meta
                            .get("content")
                            .and_then(|value| value.as_str())
                            .unwrap_or("")
                            .to_owned(),
                        score: 1.0,
                        language: meta
                            .get("language")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown")
                            .to_owned(),
                    });
                }
            }
        }
        final_results
    }

    fn handle_list_vectors(&self, collection: &str, limit: usize) -> Vec<SearchResult> {
        let mut final_results = Vec::new();
        if let Some(collection_metadata) = self.metadata_store.get(collection) {
            for (ext_id, meta_val) in collection_metadata.iter().take(limit) {
                let meta = meta_val.as_object().cloned().unwrap_or_default();
                final_results.push(SearchResult {
                    id: ext_id.clone(),
                    file_path: meta
                        .get("file_path")
                        .and_then(|value| value.as_str())
                        .unwrap_or("unknown")
                        .to_owned(),
                    start_line: meta
                        .get("start_line")
                        .and_then(serde_json::Value::as_u64)
                        .or_else(|| meta.get("line_number").and_then(serde_json::Value::as_u64))
                        .unwrap_or(0) as u32,
                    content: meta
                        .get("content")
                        .and_then(|value| value.as_str())
                        .unwrap_or("")
                        .to_owned(),
                    score: 1.0,
                    language: meta
                        .get("language")
                        .and_then(|value| value.as_str())
                        .unwrap_or("unknown")
                        .to_owned(),
                });
            }
        }
        final_results
    }
}

// =============================================================================
// Search Handlers - Similarity search
// =============================================================================

impl EdgeVecActor {
    fn handle_search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        match self.index.search(query_vector, limit, &self.storage) {
            Ok(results) => {
                let mut final_results = Vec::with_capacity(results.len());
                if let Some(collection_metadata) = self.metadata_store.get(collection) {
                    for res in results {
                        let external_id: Option<String> = self
                            .id_map
                            .iter()
                            .find(|entry| *entry.value() == res.vector_id)
                            .map(|entry| entry.key().to_owned());

                        if let Some(ext_id) = external_id
                            && let Some(meta_val) = collection_metadata.get(&ext_id)
                        {
                            let meta = meta_val.as_object().cloned().unwrap_or_default();
                            let start_line = meta
                                .get("start_line")
                                .and_then(serde_json::Value::as_u64)
                                .or_else(|| {
                                    meta.get("line_number").and_then(serde_json::Value::as_u64)
                                })
                                .unwrap_or(0) as u32;
                            final_results.push(SearchResult {
                                id: ext_id,
                                file_path: meta
                                    .get("file_path")
                                    .and_then(|value| value.as_str())
                                    .unwrap_or("unknown")
                                    .to_owned(),
                                start_line,
                                content: meta
                                    .get("content")
                                    .and_then(|value| value.as_str())
                                    .unwrap_or("")
                                    .to_owned(),
                                score: res.distance as f64,
                                language: meta
                                    .get("language")
                                    .and_then(|value| value.as_str())
                                    .unwrap_or("unknown")
                                    .to_owned(),
                            });
                        }
                    }
                }
                Ok(final_results)
            }
            Err(e) => Err(Error::internal(format!("Search failed: {e}"))),
        }
    }
}

// =============================================================================
// Stats Handlers - Get collection statistics
// =============================================================================

impl EdgeVecActor {
    fn handle_get_stats(&self, collection: &str) -> HashMap<String, serde_json::Value> {
        let vector_count = self.metadata_store.get(collection).map_or(0, |m| m.len());
        let mut stats = HashMap::new();
        stats.insert(
            STATS_FIELD_COLLECTION.to_owned(),
            serde_json::json!(collection),
        );
        stats.insert(
            STATS_FIELD_VECTORS_COUNT.to_owned(),
            serde_json::json!(vector_count),
        );
        stats.insert(
            "total_indexed_vectors".to_owned(),
            serde_json::json!(self.index.len()),
        );
        stats.insert(
            "dimensions".to_owned(),
            serde_json::json!(self.config.dimensions),
        );
        stats
    }
}

// =============================================================================
// Browse Handlers - List collections, files, and chunks
// =============================================================================

impl EdgeVecActor {
    fn handle_list_collections(&self) -> Vec<CollectionInfo> {
        self.metadata_store
            .iter()
            .map(|entry| {
                let name = entry.key().clone();
                let vector_count = entry.value().len() as u64;

                // Count unique file paths
                let file_paths: std::collections::HashSet<&str> = entry
                    .value()
                    .values()
                    .filter_map(|v| {
                        v.as_object()
                            .and_then(|o| o.get("file_path"))
                            .and_then(|v| v.as_str())
                    })
                    .collect();
                let file_count = file_paths.len() as u64;

                CollectionInfo::new(name, vector_count, file_count, None, "edgevec")
            })
            .collect()
    }

    fn handle_list_file_paths(&self, collection: &str, limit: usize) -> Result<Vec<FileInfo>> {
        let collection_metadata = self
            .metadata_store
            .get(collection)
            .ok_or_else(|| Error::internal(format!("Collection '{collection}' not found")))?;

        let mut file_map: HashMap<String, (u32, String)> = HashMap::new();

        for meta_val in collection_metadata.values() {
            if let Some(meta) = meta_val.as_object()
                && let Some(file_path) = meta.get("file_path").and_then(|v| v.as_str())
            {
                let language = meta
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_owned();

                let entry = file_map
                    .entry(file_path.to_owned())
                    .or_insert((0, language));
                entry.0 += 1;
            }
        }

        let files = file_map
            .into_iter()
            .take(limit)
            .map(|(path, (chunk_count, language))| FileInfo::new(path, chunk_count, language, None))
            .collect();
        Ok(files)
    }

    fn handle_get_chunks_by_file(
        &self,
        collection: &str,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        // Normalize to forward slashes for cross-platform path matching
        let normalized_query = file_path.replace('\\', "/");
        if let Some(collection_metadata) = self.metadata_store.get(collection) {
            for (ext_id, meta_val) in collection_metadata.iter() {
                if let Some(meta) = meta_val.as_object()
                    && meta
                        .get("file_path")
                        .and_then(|v| v.as_str())
                        .is_some_and(|p| p.replace('\\', "/") == normalized_query)
                {
                    let start_line = meta
                        .get("start_line")
                        .and_then(serde_json::Value::as_u64)
                        .or_else(|| meta.get("line_number").and_then(serde_json::Value::as_u64))
                        .unwrap_or(0) as u32;

                    results.push(SearchResult {
                        id: ext_id.clone(),
                        file_path: file_path.to_owned(),
                        start_line,
                        content: meta
                            .get("content")
                            .and_then(|value| value.as_str())
                            .unwrap_or("")
                            .to_owned(),
                        score: 1.0,
                        language: meta
                            .get("language")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown")
                            .to_owned(),
                    });
                }
            }
        }
        // Sort by start_line
        results.sort_by_key(|r| r.start_line);
        Ok(results)
    }
}

// =============================================================================
// Message Loop - Main actor run loop
// =============================================================================

impl EdgeVecActor {
    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                EdgeVecMessage::Core(core) => self.handle_core_message(core),
                EdgeVecMessage::Query(query) => self.handle_query_message(query),
                EdgeVecMessage::Browse(browse) => self.handle_browse_message(browse),
            }
        }
    }

    fn handle_core_message(&mut self, msg: CoreMessage) {
        match msg {
            CoreMessage::CreateCollection { name, tx } => {
                let _ = tx.send(self.handle_create_collection(name));
            }
            CoreMessage::DeleteCollection { name, tx } => {
                let _ = tx.send(self.handle_delete_collection(&name));
            }
            CoreMessage::InsertVectors {
                collection,
                vectors,
                metadata,
                tx,
            } => {
                let _ = tx.send(self.handle_insert_vectors(&collection, vectors, metadata));
            }
            CoreMessage::SearchSimilar {
                collection,
                query_vector,
                limit,
                tx,
            } => {
                let _ = tx.send(self.handle_search_similar(&collection, &query_vector, limit));
            }
            CoreMessage::DeleteVectors {
                collection,
                ids,
                tx,
            } => {
                let _ = tx.send(self.handle_delete_vectors(&collection, ids));
            }
        }
    }

    fn handle_query_message(&mut self, msg: QueryMessage) {
        match msg {
            QueryMessage::GetStats { collection, tx } => {
                let _ = tx.send(Ok(self.handle_get_stats(&collection)));
            }
            QueryMessage::ListVectors {
                collection,
                limit,
                tx,
            } => {
                let _ = tx.send(Ok(self.handle_list_vectors(&collection, limit)));
            }
            QueryMessage::GetVectorsByIds {
                collection,
                ids,
                tx,
            } => {
                let _ = tx.send(Ok(self.handle_get_vectors_by_ids(&collection, ids)));
            }
            QueryMessage::CollectionExists { name, tx } => {
                let _ = tx.send(self.handle_collection_exists(&name));
            }
        }
    }

    fn handle_browse_message(&mut self, msg: BrowseMessage) {
        match msg {
            BrowseMessage::ListCollections { tx } => {
                let _ = tx.send(Ok(self.handle_list_collections()));
            }
            BrowseMessage::ListFilePaths {
                collection,
                limit,
                tx,
            } => {
                let _ = tx.send(self.handle_list_file_paths(&collection, limit));
            }
            BrowseMessage::GetChunksByFile {
                collection,
                file_path,
                tx,
            } => {
                let _ = tx.send(self.handle_get_chunks_by_file(&collection, &file_path));
            }
        }
    }
}

// ============================================================================
// Auto-registration via linkme distributed slice
// ============================================================================

use std::sync::Arc;

use mcb_domain::registry::vector_store::{
    VECTOR_STORE_PROVIDERS, VectorStoreProviderConfig, VectorStoreProviderEntry,
};

/// Factory function for creating `EdgeVec` vector store provider instances.
fn edgevec_factory(
    config: &VectorStoreProviderConfig,
) -> std::result::Result<Arc<dyn VectorStoreProvider>, String> {
    let dimensions = config.dimensions.unwrap_or(384);
    let collection_name = config.collection.clone().ok_or_else(|| {
        "EdgeVec provider requires a collection name in vector_store config".to_owned()
    })?;
    let edgevec_config = EdgeVecConfig {
        dimensions,
        ..Default::default()
    };
    let provider = EdgeVecVectorStoreProvider::with_collection(
        &edgevec_config,
        CollectionId::from_name(&collection_name),
    )
    .map_err(|e| format!("Failed to create EdgeVec provider: {e}"))?;
    Ok(Arc::new(provider))
}

#[linkme::distributed_slice(VECTOR_STORE_PROVIDERS)]
static EDGEVEC_PROVIDER: VectorStoreProviderEntry = VectorStoreProviderEntry {
    name: "edgevec",
    description: "EdgeVec in-memory HNSW vector store (high-performance)",
    factory: edgevec_factory,
};
