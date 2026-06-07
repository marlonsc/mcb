use super::*;
use async_trait::async_trait;
use mcb_domain::error::Error;
use mcb_domain::ports::VectorStoreAdmin;
use mcb_domain::value_objects::CollectionId;
use std::collections::HashMap;

use crate::constants::{
    MILVUS_ERROR_RATE_LIMIT, PROVIDER_RETRY_BACKOFF_MS, PROVIDER_RETRY_COUNT,
    STATS_FIELD_COLLECTION, STATS_FIELD_PROVIDER, STATS_FIELD_STATUS, STATS_FIELD_VECTORS_COUNT,
    STATUS_ACTIVE,
};
use crate::utils::retry::{RetryConfig, retry_with_backoff};

#[async_trait]
impl VectorStoreAdmin for MilvusVectorStoreProvider {
    // --- Admin Methods ---

    async fn collection_exists(&self, name: &CollectionId) -> Result<bool> {
        let name_str = to_milvus_name(name);
        Self::map_milvus_error(
            self.client.has_collection(&name_str).await,
            "check collection",
        )
    }

    async fn get_stats(
        &self,
        collection: &CollectionId,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let name_str = to_milvus_name(collection);
        let stats = self
            .client
            .get_collection_stats(&name_str)
            .await
            .map_err(|e| {
                Error::vector_db(format!(
                    "Failed to get stats for collection '{collection}': {e}"
                ))
            })?;

        let mut result = HashMap::new();
        result.insert(
            STATS_FIELD_COLLECTION.to_owned(),
            serde_json::json!(collection),
        );
        result.insert(
            STATS_FIELD_STATUS.to_owned(),
            serde_json::json!(STATUS_ACTIVE),
        );

        if let Some(count_str) = stats.get("row_count")
            && let Ok(count) = count_str.parse::<i64>()
        {
            result.insert(
                STATS_FIELD_VECTORS_COUNT.to_owned(),
                serde_json::json!(count),
            );
        }

        result.insert(STATS_FIELD_PROVIDER.to_owned(), serde_json::json!("milvus"));
        Ok(result)
    }

    async fn flush(&self, collection: &CollectionId) -> Result<()> {
        let name_str = to_milvus_name(collection);
        let result = retry_with_backoff(
            RetryConfig::new(
                PROVIDER_RETRY_COUNT,
                std::time::Duration::from_millis(PROVIDER_RETRY_BACKOFF_MS),
            ),
            |_| self.client.flush_collections(vec![&name_str]),
            |e| {
                let err_str = e.to_string();
                err_str.contains(MILVUS_ERROR_RATE_LIMIT) || err_str.contains("rate limit")
            },
        )
        .await;

        result.map(|_| ()).map_err(|e| {
            let err_str = e.to_string();
            if err_str.contains(MILVUS_ERROR_RATE_LIMIT) || err_str.contains("rate limit") {
                Error::vector_db(format!("Failed to flush collection after retries: {e}"))
            } else {
                Error::vector_db(format!("Failed to flush collection: {e}"))
            }
        })
    }

    fn provider_name(&self) -> &str {
        "milvus"
    }
}
