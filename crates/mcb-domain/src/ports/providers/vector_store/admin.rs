#![allow(missing_docs)]

use async_trait::async_trait;

use super::{MetadataMap, PortResult, StoreCollectionId};

#[async_trait]
pub trait VectorStoreAdmin: Send + Sync {
    async fn collection_exists(&self, collection: &StoreCollectionId) -> PortResult<bool>;
    async fn get_stats(&self, collection: &StoreCollectionId) -> PortResult<MetadataMap>;
    async fn flush(&self, collection: &StoreCollectionId) -> PortResult<()>;
    fn provider_name(&self) -> &str;

    async fn health_check(&self) -> PortResult<()> {
        let health_check_id = StoreCollectionId::from_name("__health_check__");
        self.collection_exists(&health_check_id).await?;
        Ok(())
    }
}
