//! Context Service — Semantic Code Understanding
//!
//! Implements `ContextServiceInterface` using direct provider calls.
//! No cache layer, no wrappers — embedding + vector store only.

use std::collections::HashMap;
use std::sync::Arc;

use mcb_domain::constants::keys::{
    METADATA_KEY_CONTENT, METADATA_KEY_END_LINE, METADATA_KEY_FILE_PATH, METADATA_KEY_LANGUAGE,
    METADATA_KEY_START_LINE,
};
use mcb_domain::entities::CodeChunk;
use mcb_domain::error::Result;
use mcb_domain::ports::{ContextServiceInterface, EmbeddingProvider, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionId, Embedding, SearchResult};
use serde_json::Value;

/// Context service that delegates directly to embedding and vector store providers.
pub struct ContextServiceImpl {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextServiceImpl {
    /// Create a new context service from embedding and vector store providers.
    pub fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        Self {
            embedding_provider,
            vector_store_provider,
        }
    }
}

#[async_trait::async_trait]
impl ContextServiceInterface for ContextServiceImpl {
    async fn initialize(&self, collection: &CollectionId) -> Result<()> {
        let exists = self
            .vector_store_provider
            .collection_exists(collection)
            .await?;
        if !exists {
            let dims = self.embedding_provider.dimensions();
            self.vector_store_provider
                .create_collection(collection, dims)
                .await?;
        }
        Ok(())
    }

    async fn store_chunks(&self, collection: &CollectionId, chunks: &[CodeChunk]) -> Result<()> {
        if chunks.is_empty() {
            return Ok(());
        }

        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embedding_provider.embed_batch(&texts).await?;

        let metadata: Vec<HashMap<String, Value>> = chunks
            .iter()
            .map(|chunk| {
                let mut m = HashMap::new();
                m.insert(
                    METADATA_KEY_FILE_PATH.to_owned(),
                    Value::String(chunk.file_path.clone()),
                );
                m.insert(
                    METADATA_KEY_START_LINE.to_owned(),
                    Value::String(chunk.start_line.to_string()),
                );
                m.insert(
                    METADATA_KEY_END_LINE.to_owned(),
                    Value::String(chunk.end_line.to_string()),
                );
                m.insert(
                    METADATA_KEY_CONTENT.to_owned(),
                    Value::String(chunk.content.clone()),
                );
                if !chunk.language.is_empty() {
                    m.insert(
                        METADATA_KEY_LANGUAGE.to_owned(),
                        Value::String(chunk.language.clone()),
                    );
                }
                m
            })
            .collect();

        self.vector_store_provider
            .insert_vectors(collection, &embeddings, metadata)
            .await?;

        Ok(())
    }

    async fn search_similar(
        &self,
        collection: &CollectionId,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let embedding = self.embedding_provider.embed(query).await?;
        self.vector_store_provider
            .search_similar(collection, &embedding.vector, limit, None)
            .await
    }

    async fn embed_text(&self, text: &str) -> Result<Embedding> {
        self.embedding_provider.embed(text).await
    }

    async fn clear_collection(&self, collection: &CollectionId) -> Result<()> {
        self.vector_store_provider
            .delete_collection(collection)
            .await
    }

    async fn get_stats(&self) -> Result<(i64, i64)> {
        Ok((0, 0))
    }

    fn embedding_dimensions(&self) -> usize {
        self.embedding_provider.dimensions()
    }
}

// ---------------------------------------------------------------------------
// Linkme Registration
// ---------------------------------------------------------------------------
use mcb_domain::registry::services::{
    CONTEXT_SERVICE_NAME, SERVICES_REGISTRY, ServiceBuilder, ServiceRegistryEntry,
};

#[linkme::distributed_slice(SERVICES_REGISTRY)]
static CONTEXT_SERVICE_REGISTRY_ENTRY: ServiceRegistryEntry = ServiceRegistryEntry {
    name: CONTEXT_SERVICE_NAME,
    build: ServiceBuilder::Context(|context| {
        let ctx = context
            .downcast_ref::<mcb_domain::registry::ServiceResolutionContext>()
            .ok_or_else(|| {
                mcb_domain::error::Error::internal(
                    "Context service builder requires ServiceResolutionContext",
                )
            })?;

        let embedding = Arc::clone(&ctx.embedding_provider);
        let vector_store = Arc::clone(&ctx.vector_store_provider);

        Ok(Arc::new(ContextServiceImpl::new(embedding, vector_store)))
    }),
};
