//! Admin operations for Weaviate vector store provider.

use std::collections::HashMap;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::VectorStoreAdmin;
use mcb_domain::value_objects::CollectionId;
use serde_json::Value;

use mcb_utils::constants::vector_store::{
    STATS_FIELD_COLLECTION, STATS_FIELD_PROVIDER, STATS_FIELD_STATUS, STATS_FIELD_VECTORS_COUNT,
    STATUS_ACTIVE, STATUS_UNKNOWN,
};

use super::WeaviateVectorStoreProvider;

#[async_trait]
impl VectorStoreAdmin for WeaviateVectorStoreProvider {
    async fn collection_exists(&self, name: &CollectionId) -> Result<bool> {
        let class = Self::class_name(name);
        match self
            .request(reqwest::Method::GET, &format!("/v1/schema/{class}"), None)
            .await
        {
            Ok(_) => Ok(true),
            // A missing class is a legitimate "does not exist" answer, not a
            // transport failure: Weaviate returns 404 for unknown classes.
            Err(_) => Ok(false),
        }
    }

    async fn get_stats(&self, collection: &CollectionId) -> Result<HashMap<String, Value>> {
        let class = Self::class_name(collection);
        let query = format!("{{ Aggregate {{ {class} {{ meta {{ count }} }} }} }}");

        let response = self
            .request(
                reqwest::Method::POST,
                "/v1/graphql",
                Some(serde_json::json!({ "query": query })),
            )
            .await;

        let mut stats = HashMap::new();
        stats.insert(
            STATS_FIELD_COLLECTION.to_owned(),
            serde_json::json!(collection.to_string()),
        );
        stats.insert(
            STATS_FIELD_PROVIDER.to_owned(),
            serde_json::json!(self.provider_name()),
        );

        let (status, count) = match response {
            Ok(data) => (
                STATUS_ACTIVE,
                data.get("data")
                    .and_then(|d| d.get("Aggregate"))
                    .and_then(|a| a.get(&class))
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|m| m.get("meta"))
                    .and_then(|m| m.get("count"))
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
        // Weaviate writes are durable/consistent once acknowledged.
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "weaviate"
    }
}
