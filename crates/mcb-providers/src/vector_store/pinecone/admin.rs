//! Admin operations for Pinecone vector store provider.

use std::collections::HashMap;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::VectorStoreAdmin;
use mcb_domain::value_objects::CollectionId;
use serde_json::Value;

use crate::constants::{
    STATS_FIELD_COLLECTION, STATS_FIELD_PROVIDER, STATS_FIELD_STATUS, STATS_FIELD_VECTORS_COUNT,
    STATUS_ACTIVE, STATUS_UNKNOWN,
};

use super::PineconeVectorStoreProvider;

#[async_trait]
impl VectorStoreAdmin for PineconeVectorStoreProvider {
    // --- Admin Methods ---

    async fn collection_exists(&self, name: &CollectionId) -> Result<bool> {
        let name_str = name.to_string();
        Ok(self.collections.contains_key(&name_str))
    }

    async fn get_stats(&self, collection: &CollectionId) -> Result<HashMap<String, Value>> {
        let collection_str = collection.to_string();
        let response = self
            .request(
                reqwest::Method::POST,
                "/describe_index_stats",
                Some(serde_json::json!({ "filter": {} })),
            )
            .await;

        let mut stats = HashMap::new();
        stats.insert(
            STATS_FIELD_COLLECTION.to_owned(),
            serde_json::json!(&collection_str),
        );
        stats.insert(
            STATS_FIELD_PROVIDER.to_owned(),
            serde_json::json!(self.provider_name()),
        );

        let (status, count) = match response {
            Ok(data) => (
                STATUS_ACTIVE,
                data.get("namespaces")
                    .and_then(|namespaces| namespaces.get(&collection_str))
                    .and_then(|ns| ns.get("vectorCount"))
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!(0)),
            ),
            Err(_) => (STATUS_UNKNOWN, serde_json::json!(0)),
        };
        stats.insert(STATS_FIELD_STATUS.to_owned(), serde_json::json!(status));
        stats.insert(STATS_FIELD_VECTORS_COUNT.to_owned(), count);

        Ok(stats)
    }

    async fn flush(&self, _collection: &CollectionId) -> Result<()> {
        // Pinecone writes are immediately consistent
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "pinecone"
    }
}
