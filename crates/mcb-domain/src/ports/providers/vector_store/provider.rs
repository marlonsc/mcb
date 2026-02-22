//!
//! **Documentation**: [docs/modules/domain.md](../../../../../../docs/modules/domain.md#provider-ports)
//!
#![allow(missing_docs)]

use async_trait::async_trait;

use super::{
    MetadataMap, PortResult, StoreCollectionId, StoreEmbedding, StoreSearchResult,
    VectorStoreAdmin, VectorStoreBrowser,
};

#[async_trait]
pub trait VectorStoreProvider: VectorStoreAdmin + VectorStoreBrowser + Send + Sync {
    async fn create_collection(
        &self,
        collection: &StoreCollectionId,
        dimensions: usize,
    ) -> PortResult<()>;

    async fn delete_collection(&self, collection: &StoreCollectionId) -> PortResult<()>;

    async fn insert_vectors(
        &self,
        collection: &StoreCollectionId,
        vectors: &[StoreEmbedding],
        metadata: Vec<MetadataMap>,
    ) -> PortResult<Vec<String>>;

    async fn search_similar(
        &self,
        collection: &StoreCollectionId,
        query_vector: &[f32],
        limit: usize,
        filter: Option<&str>,
    ) -> PortResult<Vec<StoreSearchResult>>;

    async fn delete_vectors(
        &self,
        collection: &StoreCollectionId,
        ids: &[String],
    ) -> PortResult<()>;

    async fn get_vectors_by_ids(
        &self,
        collection: &StoreCollectionId,
        ids: &[String],
    ) -> PortResult<Vec<StoreSearchResult>>;

    async fn list_vectors(
        &self,
        collection: &StoreCollectionId,
        limit: usize,
    ) -> PortResult<Vec<StoreSearchResult>>;
}
