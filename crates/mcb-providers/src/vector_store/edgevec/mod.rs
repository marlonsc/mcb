//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#vector-store-providers)
//!
//! `EdgeVec` Vector Store Provider
//!
//! High-performance embedded vector database implementation using `EdgeVec`.
//! `EdgeVec` provides sub-millisecond vector similarity search with HNSW algorithm.
//! This implementation uses the Actor pattern to eliminate locks and ensure non-blocking operation.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use edgevec::hnsw::VectorId;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::utils::id;
use mcb_domain::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};
use tokio::sync::{mpsc, oneshot};

use crate::constants::{
    EDGEVEC_DEFAULT_DIMENSIONS, EDGEVEC_HNSW_EF_CONSTRUCTION, EDGEVEC_HNSW_EF_SEARCH,
    EDGEVEC_HNSW_M, EDGEVEC_HNSW_M0, EDGEVEC_QUANTIZATION_TYPE, STATS_FIELD_COLLECTION,
    STATS_FIELD_VECTORS_COUNT, VECTOR_FIELD_FILE_PATH, VECTOR_FIELD_LANGUAGE,
};
use crate::utils::vector_store::search_result_from_json_metadata;

mod actor;
mod config;
mod provider;

pub use config::{EdgeVecConfig, HnswConfig, MetricType, QuantizerConfig};

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

type CollectionMetadata = HashMap<String, serde_json::Value>;

/// `EdgeVec` vector store provider implementation using Actor pattern
pub struct EdgeVecVectorStoreProvider {
    sender: mpsc::Sender<EdgeVecMessage>,
    _collection: CollectionId,
}

impl EdgeVecVectorStoreProvider {
    async fn send_message<T, F>(&self, build_message: F) -> Result<T>
    where
        F: FnOnce(oneshot::Sender<Result<T>>) -> EdgeVecMessage,
    {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(build_message(tx))
            .await
            .map_err(|_| Error::vector_db("Actor channel closed"))?;
        rx.await
            .map_err(|_| Error::vector_db("Actor response channel closed"))?
    }
    async fn send_core<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(oneshot::Sender<Result<T>>) -> CoreMessage,
    {
        self.send_message(|tx| EdgeVecMessage::Core(f(tx))).await
    }
    async fn send_query<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(oneshot::Sender<Result<T>>) -> QueryMessage,
    {
        self.send_message(|tx| EdgeVecMessage::Query(f(tx))).await
    }
    async fn send_browse<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(oneshot::Sender<Result<T>>) -> BrowseMessage,
    {
        self.send_message(|tx| EdgeVecMessage::Browse(f(tx))).await
    }

    /// Create a new `EdgeVec` vector store provider
    ///
    /// # Errors
    ///
    /// Returns an error if the `EdgeVec` actor fails to initialize.
    pub fn new(config: &EdgeVecConfig) -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);
        let config_clone = config.clone();

        let actor = actor::EdgeVecActor::new(rx, config_clone)?;
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

        let actor = actor::EdgeVecActor::new(rx, config_clone)?;
        tokio::spawn(async move {
            actor.run().await;
        });

        Ok(Self {
            sender: tx,
            _collection: collection,
        })
    }
}

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
    build: edgevec_factory,
};
