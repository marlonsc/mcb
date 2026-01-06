//! Business logic services

use crate::core::types::{CodeChunk, SearchResult};
use crate::factory::ServiceProvider;
use crate::providers::{EmbeddingProvider, VectorStoreProvider};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

/// Context service for managing embeddings and vector storage
pub struct ContextService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_store_provider: Arc<dyn VectorStoreProvider>,
}

impl ContextService {
    pub fn new(service_provider: &ServiceProvider) -> crate::core::error::Result<Self> {
        // For MVP, use default providers
        let embedding_config = crate::core::types::EmbeddingConfig {
            provider: "mock".to_string(),
            model: "mock".to_string(),
            api_key: None,
            base_url: None,
            dimensions: Some(128),
            max_tokens: Some(512),
        };

        let vector_store_config = crate::core::types::VectorStoreConfig {
            provider: "in-memory".to_string(),
            address: None,
            token: None,
            collection: None,
            dimensions: Some(128),
        };

        // This would normally be async, but for simplicity in MVP we create synchronously
        // In production, this would be async and use the service provider properly
        let embedding_provider = Arc::new(crate::providers::MockEmbeddingProvider::new());
        let vector_store_provider = Arc::new(crate::providers::InMemoryVectorStoreProvider::new());

        Ok(Self {
            embedding_provider,
            vector_store_provider,
        })
    }

    pub async fn embed_text(&self, text: &str) -> crate::core::error::Result<crate::core::types::Embedding> {
        self.embedding_provider.embed(text).await
    }

    pub async fn store_chunks(&self, collection: &str, chunks: &[CodeChunk]) -> crate::core::error::Result<()> {
        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let embeddings = self.embedding_provider.embed_batch(&texts).await?;
        self.vector_store_provider.store(collection, &embeddings).await
    }

    pub async fn search_similar(&self, collection: &str, query: &str, limit: usize) -> crate::core::error::Result<Vec<SearchResult>> {
        let query_embedding = self.embed_text(query).await?;
        let results = self.vector_store_provider.search(collection, &query_embedding.vector, limit).await?;

        let search_results = results.into_iter().map(|(score, _embedding)| {
            SearchResult {
                file_path: "unknown".to_string(),
                line_number: 0,
                content: query.to_string(),
                score,
                metadata: serde_json::json!({}),
            }
        }).collect();

        Ok(search_results)
    }
}

/// Indexing service for processing codebases
pub struct IndexingService {
    context_service: Arc<ContextService>,
}

impl IndexingService {
    pub fn new(context_service: Arc<ContextService>) -> Self {
        Self { context_service }
    }

    pub async fn index_directory(&self, path: &Path, collection: &str) -> crate::core::error::Result<usize> {
        // Simple MVP implementation
        Ok(0)
    }
}

/// Search service for querying indexed code
pub struct SearchService {
    context_service: Arc<ContextService>,
}

impl SearchService {
    pub fn new(context_service: Arc<ContextService>) -> Self {
        Self { context_service }
    }

    pub async fn search(&self, collection: &str, query: &str, limit: usize) -> crate::core::error::Result<Vec<SearchResult>> {
        self.context_service.search_similar(collection, query, limit).await
    }
}