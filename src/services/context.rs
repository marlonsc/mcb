//! Code Intelligence Business Service
//!
//! The Context Service transforms raw code into semantic understanding through
//! AI embeddings and intelligent storage. This business service powers the core
//! intelligence behind semantic code search, enabling development teams to find
//! code by meaning rather than keywords.

use crate::core::error::{Error, Result};
use crate::core::hybrid_search::{HybridSearchActor, HybridSearchConfig, HybridSearchMessage};
use crate::core::types::{CodeChunk, Embedding, SearchResult};
use crate::providers::{EmbeddingProvider, VectorStoreProvider};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

/// Enterprise Code Intelligence Coordinator
pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
    hybrid_search_sender: mpsc::Sender<HybridSearchMessage>,
}

impl ContextService {
    /// Create a new context service with specified providers
    pub fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        vector_store_provider: Arc<dyn VectorStoreProvider>,
    ) -> Self {
        let config = HybridSearchConfig::from_env();
        let (bm25_weight, semantic_weight) = if config.enabled {
            (config.bm25_weight, config.semantic_weight)
        } else {
            (0.0, 1.0)
        };

        let (sender, receiver) = mpsc::channel(100);
        let actor = HybridSearchActor::new(receiver, bm25_weight, semantic_weight);

        // Start the hybrid search actor in the background
        tokio::spawn(async move {
            actor.run().await;
        });

        Self {
            embedding_provider,
            vector_store_provider,
            hybrid_search_sender: sender,
        }
    }

    /// Generate embeddings for text
    pub async fn embed_text(&self, text: &str) -> Result<Embedding> {
        self.embedding_provider.embed(text).await
    }

    /// Generate embeddings for multiple texts
    pub async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        self.embedding_provider.embed_batch(texts).await
    }

    /// Store code chunks in vector database
    pub async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embed_texts(&texts).await?;

        // Prepare metadata for each chunk
        let metadata: Vec<HashMap<String, serde_json::Value>> = chunks
            .iter()
            .map(|chunk| {
                let mut meta = HashMap::new();
                meta.insert("content".to_string(), serde_json::json!(chunk.content));
                meta.insert("file_path".to_string(), serde_json::json!(chunk.file_path));
                meta.insert(
                    "start_line".to_string(),
                    serde_json::json!(chunk.start_line),
                );
                meta.insert("end_line".to_string(), serde_json::json!(chunk.end_line));
                meta.insert(
                    "language".to_string(),
                    serde_json::json!(format!("{:?}", chunk.language)),
                );
                meta
            })
            .collect();

        // Ensure collection exists
        if !self
            .vector_store_provider
            .collection_exists(collection)
            .await?
        {
            self.vector_store_provider
                .create_collection(collection, self.embedding_dimensions())
                .await?;
        }

        self.vector_store_provider
            .insert_vectors(collection, &embeddings, metadata)
            .await?;

        // Index documents for hybrid search (BM25) via Actor
        self.hybrid_search_sender
            .send(HybridSearchMessage::Index {
                collection: collection.to_string(),
                chunks: chunks.to_vec(),
            })
            .await
            .map_err(|e| {
                Error::internal(format!("Failed to send to hybrid search actor: {}", e))
            })?;

        Ok(())
    }

    /// Search for similar code chunks using hybrid search (BM25 + semantic embeddings)
    pub async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embed_text(query).await?;

        // Get semantic search results
        let expanded_limit = (limit * 2).clamp(20, 100);
        let semantic_results = self
            .vector_store_provider
            .search_similar(collection, &query_embedding.vector, expanded_limit, None)
            .await?;

        let semantic_search_results: Vec<SearchResult> = semantic_results
            .into_iter()
            .map(|result| SearchResult {
                file_path: result
                    .metadata
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: result
                    .metadata
                    .get("start_line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                content: result
                    .metadata
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                score: result.score,
                metadata: result.metadata,
            })
            .collect();

        // Request hybrid search from Actor
        let (respond_to, receiver) = oneshot::channel();
        self.hybrid_search_sender
            .send(HybridSearchMessage::Search {
                query: query.to_string(),
                semantic_results: semantic_search_results,
                limit,
                respond_to,
            })
            .await
            .map_err(|e| {
                Error::internal(format!(
                    "Failed to send search to hybrid search actor: {}",
                    e
                ))
            })?;

        let hybrid_results = receiver.await.map_err(|e| {
            Error::internal(format!("Failed to receive hybrid search results: {}", e))
        })??;

        Ok(hybrid_results
            .into_iter()
            .map(|hybrid_result| {
                let mut result = hybrid_result.result;
                result.score = hybrid_result.hybrid_score;

                let mut new_metadata = serde_json::Map::new();
                if let serde_json::Value::Object(existing) = &result.metadata {
                    new_metadata.extend(existing.clone());
                }
                new_metadata.insert(
                    "bm25_score".to_string(),
                    serde_json::json!(hybrid_result.bm25_score),
                );
                new_metadata.insert(
                    "semantic_score".to_string(),
                    serde_json::json!(hybrid_result.semantic_score),
                );
                new_metadata.insert(
                    "hybrid_score".to_string(),
                    serde_json::json!(hybrid_result.hybrid_score),
                );
                result.metadata = serde_json::Value::Object(new_metadata);

                result
            })
            .collect())
    }

    /// Clear a collection
    pub async fn clear_collection(&self, collection: &str) -> Result<()> {
        self.vector_store_provider
            .delete_collection(collection)
            .await?;

        self.hybrid_search_sender
            .send(HybridSearchMessage::Clear {
                collection: collection.to_string(),
            })
            .await
            .map_err(|e| {
                Error::internal(format!(
                    "Failed to send clear to hybrid search actor: {}",
                    e
                ))
            })?;

        Ok(())
    }

    /// Get embedding dimensions
    pub fn embedding_dimensions(&self) -> usize {
        self.embedding_provider.dimensions()
    }

    /// Get hybrid search statistics
    pub async fn get_hybrid_search_stats(&self) -> HashMap<String, serde_json::Value> {
        let (respond_to, receiver) = oneshot::channel();
        if self
            .hybrid_search_sender
            .send(HybridSearchMessage::GetStats { respond_to })
            .await
            .is_err()
        {
            return HashMap::new();
        }

        receiver.await.unwrap_or_default()
    }
}

/// Generic context service using Strategy pattern with trait bounds
pub struct GenericContextService<E, V>
where
    E: EmbeddingProvider + Send + Sync,
    V: VectorStoreProvider + Send + Sync,
{
    embedding_provider: Arc<E>,
    vector_store_provider: Arc<V>,
    hybrid_search_sender: mpsc::Sender<HybridSearchMessage>,
}

impl<E, V> GenericContextService<E, V>
where
    E: EmbeddingProvider + Send + Sync,
    V: VectorStoreProvider + Send + Sync,
{
    /// Create a new generic context service with specified provider strategies
    pub fn new(embedding_provider: Arc<E>, vector_store_provider: Arc<V>) -> Self {
        let config = HybridSearchConfig::from_env();
        let (bm25_weight, semantic_weight) = if config.enabled {
            (config.bm25_weight, config.semantic_weight)
        } else {
            (0.0, 1.0)
        };

        let (sender, receiver) = mpsc::channel(100);
        let actor = HybridSearchActor::new(receiver, bm25_weight, semantic_weight);

        tokio::spawn(async move {
            actor.run().await;
        });

        Self {
            embedding_provider,
            vector_store_provider,
            hybrid_search_sender: sender,
        }
    }

    /// Generate embeddings for text
    pub async fn embed_text(&self, text: &str) -> Result<Embedding> {
        self.embedding_provider.embed(text).await
    }

    /// Generate embeddings for multiple texts
    pub async fn embed_texts(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        self.embedding_provider.embed_batch(texts).await
    }

    /// Search for similar code chunks
    pub async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedding_provider.embed(query).await?;

        let expanded_limit = (limit * 2).clamp(20, 100);
        let semantic_results = self
            .vector_store_provider
            .search_similar(collection, &query_embedding.vector, expanded_limit, None)
            .await?;

        let semantic_search_results: Vec<SearchResult> = semantic_results
            .into_iter()
            .map(|result| SearchResult {
                file_path: result
                    .metadata
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                line_number: result
                    .metadata
                    .get("start_line")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                content: result
                    .metadata
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                score: result.score,
                metadata: result.metadata,
            })
            .collect();

        let (respond_to, receiver) = oneshot::channel();
        self.hybrid_search_sender
            .send(HybridSearchMessage::Search {
                query: query.to_string(),
                semantic_results: semantic_search_results,
                limit,
                respond_to,
            })
            .await
            .map_err(|e| {
                Error::internal(format!(
                    "Failed to send search to hybrid search actor: {}",
                    e
                ))
            })?;

        let hybrid_results = receiver.await.map_err(|e| {
            Error::internal(format!("Failed to receive hybrid search results: {}", e))
        })??;

        Ok(hybrid_results
            .into_iter()
            .map(|hybrid_result| {
                let mut result = hybrid_result.result;
                result.score = hybrid_result.hybrid_score;

                let mut new_metadata = serde_json::Map::new();
                if let serde_json::Value::Object(existing) = &result.metadata {
                    new_metadata.extend(existing.clone());
                }
                new_metadata.insert(
                    "bm25_score".to_string(),
                    serde_json::json!(hybrid_result.bm25_score),
                );
                new_metadata.insert(
                    "semantic_score".to_string(),
                    serde_json::json!(hybrid_result.semantic_score),
                );
                new_metadata.insert(
                    "hybrid_score".to_string(),
                    serde_json::json!(hybrid_result.hybrid_score),
                );
                result.metadata = serde_json::Value::Object(new_metadata);

                result
            })
            .collect())
    }

    /// Get embedding dimensions
    pub fn embedding_dimensions(&self) -> usize {
        self.embedding_provider.dimensions()
    }

    /// Get hybrid search statistics
    pub async fn get_hybrid_search_stats(&self) -> HashMap<String, serde_json::Value> {
        let (respond_to, receiver) = oneshot::channel();
        if self
            .hybrid_search_sender
            .send(HybridSearchMessage::GetStats { respond_to })
            .await
            .is_err()
        {
            return HashMap::new();
        }

        receiver.await.unwrap_or_default()
    }
}

/// Repository-based context service using Repository pattern
pub struct RepositoryContextService<C, S>
where
    C: crate::repository::ChunkRepository + Send + Sync,
    S: crate::repository::SearchRepository + Send + Sync,
{
    chunk_repository: Arc<C>,
    search_repository: Arc<S>,
}

impl<C, S> RepositoryContextService<C, S>
where
    C: crate::repository::ChunkRepository + Send + Sync,
    S: crate::repository::SearchRepository + Send + Sync,
{
    /// Create a new repository-based context service
    pub fn new(chunk_repository: Arc<C>, search_repository: Arc<S>) -> Self {
        Self {
            chunk_repository,
            search_repository,
        }
    }

    /// Generate embeddings for text using repository-based approach
    pub async fn embed_text(&self, _text: &str) -> Result<Embedding> {
        Err(Error::generic("Repository-based embedding not implemented"))
    }

    /// Store code chunks using the chunk repository
    pub async fn store_chunks(&self, _collection: &str, chunks: &[CodeChunk]) -> Result<()> {
        self.chunk_repository.save_batch(chunks).await?;
        self.search_repository
            .index_for_hybrid_search(chunks)
            .await?;
        Ok(())
    }

    /// Search for similar code chunks using repository-based search
    pub async fn search_similar(
        &self,
        collection: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let query_vector = vec![0.0f32; 384]; // Mock dimension
        self.search_repository
            .hybrid_search(collection, query, &query_vector, limit)
            .await
    }

    /// Clear a collection using repositories
    pub async fn clear_collection(&self, collection: &str) -> Result<()> {
        self.chunk_repository.delete_collection(collection).await?;
        self.search_repository.clear_index(collection).await?;
        Ok(())
    }

    /// Get repository statistics
    pub async fn get_repository_stats(
        &self,
    ) -> Result<(
        crate::repository::RepositoryStats,
        crate::repository::SearchStats,
    )> {
        let chunk_stats = self.chunk_repository.stats().await?;
        let search_stats = self.search_repository.search_stats().await?;
        Ok((chunk_stats, search_stats))
    }
}

impl Default for ContextService {
    fn default() -> Self {
        let embedding_provider = Arc::new(crate::providers::MockEmbeddingProvider::new());
        let vector_store_provider = Arc::new(crate::providers::InMemoryVectorStoreProvider::new());
        Self::new(embedding_provider, vector_store_provider)
    }
}
