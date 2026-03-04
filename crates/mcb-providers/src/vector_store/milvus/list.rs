use super::*;
use mcb_domain::value_objects::{CollectionId, SearchResult};

use mcb_utils::constants::vector_store::MILVUS_QUERY_BATCH_SIZE;

impl MilvusVectorStoreProvider {
    pub(super) async fn fetch_list_vectors_batch(
        &self,
        collection: &CollectionId,
        offset: i64,
        remaining: usize,
        current_total: usize,
    ) -> Result<Option<Vec<milvus::data::FieldColumn>>> {
        use milvus::query::QueryOptions;

        let batch_limit = remaining.min(MILVUS_QUERY_BATCH_SIZE) as i64;
        let query_options = QueryOptions::new()
            .limit(batch_limit)
            .offset(offset)
            .output_fields(Self::default_output_fields());

        match self
            .client
            .query(to_milvus_name(collection), "id >= 0", &query_options)
            .await
        {
            Ok(results) => Ok(Some(results)),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("message length too large") {
                    mcb_domain::warn!(
                        "milvus",
                        "hit gRPC message size limit, returning partial results",
                        &format!("offset = {offset}, results = {current_total}")
                    );
                    return Ok(None);
                }
                Err(mcb_domain::error::Error::vector_db(format!(
                    "Failed to list vectors: {e}"
                )))
            }
        }
    }

    async fn ensure_collection_loaded(&self, collection: &CollectionId) -> Result<()> {
        let name_str = to_milvus_name(collection);
        self.client
            .load_collection(&name_str, None)
            .await
            .map_err(|e| {
                mcb_domain::error::Error::vector_db(format!(
                    "Failed to load collection '{collection}': {e}"
                ))
            })?;
        Ok(())
    }

    async fn collect_list_vectors_batches(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut all_results = Vec::new();
        let mut offset = 0i64;

        loop {
            let remaining = limit.saturating_sub(all_results.len());
            if remaining == 0 {
                break;
            }

            let Some(query_results) = self
                .fetch_list_vectors_batch(collection, offset, remaining, all_results.len())
                .await?
            else {
                break;
            };

            let row_count = browser::query_row_count(&query_results);
            if row_count == 0 {
                break;
            }

            all_results.extend(browser::convert_query_results(&query_results, None)?);

            offset += row_count as i64;

            if row_count < remaining.min(MILVUS_QUERY_BATCH_SIZE) {
                break;
            }
        }

        Ok(all_results)
    }

    pub(super) async fn list_vectors_impl(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        self.ensure_collection_loaded(collection).await?;
        self.collect_list_vectors_batches(collection, limit).await
    }
}
