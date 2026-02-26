//!
//! **Documentation**: [docs/modules/domain.md](../../../../../../docs/modules/domain.md#provider-ports)
//!
#![allow(missing_docs)]

use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;

use super::{VectorStoreAdmin, VectorStoreBrowser};
use crate::error::Result;
use crate::value_objects::{CollectionId, Embedding, SearchResult};

#[async_trait]
pub trait VectorStoreProvider: VectorStoreAdmin + VectorStoreBrowser + Send + Sync {
    async fn create_collection(&self, collection: &CollectionId, dimensions: usize) -> Result<()>;

    async fn delete_collection(&self, collection: &CollectionId) -> Result<()>;

    async fn insert_vectors(
        &self,
        collection: &CollectionId,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, Value>>,
    ) -> Result<Vec<String>>;

    async fn search_similar(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> Result<Vec<SearchResult>>;

    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()>;

    async fn get_vectors_by_ids(
        &self,
        collection: &CollectionId,
        ids: &[String],
    ) -> Result<Vec<SearchResult>>;

    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
}
