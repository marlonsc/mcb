//! `VectorStoreAdmin` implementation for Qdrant.

use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;

use mcb_domain::error::Result;
use mcb_domain::ports::VectorStoreAdmin;
use mcb_domain::value_objects::CollectionId;

use crate::constants::{
    STATS_FIELD_COLLECTION, STATS_FIELD_PROVIDER, STATS_FIELD_STATUS, STATS_FIELD_VECTORS_COUNT,
    STATUS_UNKNOWN,
};

use super::QdrantVectorStoreProvider;

#[async_trait]
impl VectorStoreAdmin for QdrantVectorStoreProvider {
    async fn collection_exists(&self, name: &CollectionId) -> Result<bool> {
        match self
            .request_collection(reqwest::Method::GET, name, None)
            .await
        {
            Ok(_) => Ok(true),
            Err(e) if e.to_string().contains("(404)") => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn get_stats(&self, collection: &CollectionId) -> Result<HashMap<String, Value>> {
        let mut stats = HashMap::new();
        stats.insert(
            STATS_FIELD_COLLECTION.to_owned(),
            serde_json::json!(collection.to_string()),
        );
        stats.insert(
            STATS_FIELD_PROVIDER.to_owned(),
            serde_json::json!(self.provider_name()),
        );

        match self
            .request_collection(reqwest::Method::GET, collection, None)
            .await
        {
            Ok(data) => {
                if let Some(result) = data.get("result") {
                    if let Some(count) = result.get(STATS_FIELD_VECTORS_COUNT) {
                        stats.insert(STATS_FIELD_VECTORS_COUNT.to_owned(), count.clone());
                    }
                    if let Some(status) = result.get("status") {
                        stats.insert(STATS_FIELD_STATUS.to_owned(), status.clone());
                    }
                }
            }
            Err(_) => {
                stats.insert(
                    STATS_FIELD_STATUS.to_owned(),
                    serde_json::json!(STATUS_UNKNOWN),
                );
                stats.insert(STATS_FIELD_VECTORS_COUNT.to_owned(), serde_json::json!(0));
            }
        }

        Ok(stats)
    }

    async fn flush(&self, _collection: &CollectionId) -> Result<()> {
        // Qdrant handles persistence automatically
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "qdrant"
    }
}
