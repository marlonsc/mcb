//! EdgeVec vector store client types and implementation.

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use edgevec::hnsw::VectorId;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};
use mcb_utils::utils::id;
use tokio::sync::{mpsc, oneshot};

use super::actor;
use super::config::EdgeVecConfig;

/// Core collection management messages
pub(super) enum CoreMessage {
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
pub(super) enum QueryMessage {
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
pub(super) enum BrowseMessage {
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
pub(super) enum EdgeVecMessage {
    Core(CoreMessage),
    Query(QueryMessage),
    Browse(BrowseMessage),
}

pub(super) type CollectionMetadata = HashMap<String, serde_json::Value>;

/// `EdgeVec` vector store provider implementation using Actor pattern
pub struct EdgeVecVectorStoreProvider {
    pub(super) sender: mpsc::Sender<EdgeVecMessage>,
    pub(super) _collection: CollectionId,
}

impl EdgeVecVectorStoreProvider {
    pub(super) async fn send_message<T, F>(&self, build_message: F) -> Result<T>
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
    pub(super) async fn send_core<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(oneshot::Sender<Result<T>>) -> CoreMessage,
    {
        self.send_message(|tx| EdgeVecMessage::Core(f(tx))).await
    }
    pub(super) async fn send_query<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(oneshot::Sender<Result<T>>) -> QueryMessage,
    {
        self.send_message(|tx| EdgeVecMessage::Query(f(tx))).await
    }
    pub(super) async fn send_browse<T, F>(&self, f: F) -> Result<T>
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
        let (tx, rx) = mpsc::channel(mcb_utils::constants::vector_store::EDGEVEC_CHANNEL_CAPACITY);
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
        let (tx, rx) = mpsc::channel(mcb_utils::constants::vector_store::EDGEVEC_CHANNEL_CAPACITY);
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
