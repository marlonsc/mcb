//!
//! **Documentation**: [docs/modules/domain.md](../../../../../../docs/modules/domain.md#provider-ports)
//!
#![allow(missing_docs)]

use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;

use crate::error::Result;
use crate::value_objects::CollectionId;

#[async_trait]
pub trait VectorStoreAdmin: Send + Sync {
    async fn collection_exists(&self, collection: &CollectionId) -> Result<bool>;
    async fn get_stats(&self, collection: &CollectionId) -> Result<HashMap<String, Value>>;
    async fn flush(&self, collection: &CollectionId) -> Result<()>;
    fn provider_name(&self) -> &str;

    async fn health_check(&self) -> Result<()> {
        let health_check_id = CollectionId::from_name("__health_check__");
        self.collection_exists(&health_check_id).await?;
        Ok(())
    }
}
