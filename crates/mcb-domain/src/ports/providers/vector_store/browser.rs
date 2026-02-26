//!
//! **Documentation**: [docs/modules/domain.md](../../../../../../docs/modules/domain.md#provider-ports)
//!
#![allow(missing_docs)]

use async_trait::async_trait;

use crate::error::Result;
use crate::value_objects::{CollectionId, CollectionInfo, FileInfo, SearchResult};

#[async_trait]
pub trait VectorStoreBrowser: Send + Sync {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>>;

    async fn list_file_paths(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<FileInfo>>;

    async fn get_chunks_by_file(
        &self,
        collection: &CollectionId,
        file_path: &str,
    ) -> Result<Vec<SearchResult>>;
}
