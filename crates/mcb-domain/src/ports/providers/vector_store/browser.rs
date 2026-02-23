//!
//! **Documentation**: [docs/modules/domain.md](../../../../../../docs/modules/domain.md#provider-ports)
//!
#![allow(missing_docs)]

use async_trait::async_trait;

use super::{PortResult, StoreCollectionId, StoreCollectionInfo, StoreFileInfo, StoreSearchResult};

#[async_trait]
pub trait VectorStoreBrowser: Send + Sync {
    async fn list_collections(&self) -> PortResult<Vec<StoreCollectionInfo>>;

    async fn list_file_paths(
        &self,
        collection: &StoreCollectionId,
        limit: usize,
    ) -> PortResult<Vec<StoreFileInfo>>;

    async fn get_chunks_by_file(
        &self,
        collection: &StoreCollectionId,
        file_path: &str,
    ) -> PortResult<Vec<StoreSearchResult>>;
}
