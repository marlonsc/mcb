//! Browser operations for Weaviate vector store provider.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionId, CollectionInfo, FileInfo, SearchResult};
use serde_json::Value;

use mcb_utils::constants::vector_store::{VECTOR_FIELD_FILE_PATH, WEAVIATE_BATCH_SIZE};

use super::WeaviateVectorStoreProvider;

#[async_trait]
impl VectorStoreBrowser for WeaviateVectorStoreProvider {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let response = self
            .request(reqwest::Method::GET, "/v1/schema", None)
            .await?;

        let collections = response
            .get("classes")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.get("class").and_then(Value::as_str))
                    .map(|name| CollectionInfo::new(name, 0, 0, None, self.provider_name()))
                    .collect()
            })
            .unwrap_or_default();
        Ok(collections)
    }

    async fn list_file_paths(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<FileInfo>> {
        let results = self.list_vectors(collection, limit).await?;
        Ok(crate::utils::vector_store::build_file_info_from_results(
            results,
        ))
    }

    async fn get_chunks_by_file(
        &self,
        collection: &CollectionId,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        let class = Self::class_name(collection);
        let filter = serde_json::json!({ (VECTOR_FIELD_FILE_PATH): { "$eq": file_path } });
        let where_clause = Self::where_clause(&filter.to_string())?;
        let query = Self::build_get_query(&class, None, WEAVIATE_BATCH_SIZE, Some(&where_clause))?;

        let mut results = self.run_get_query(&class, query).await?;
        results.sort_by_key(|r| r.start_line);
        Ok(results)
    }
}
