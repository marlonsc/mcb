//! `VectorStoreBrowser` implementation for Qdrant.

use async_trait::async_trait;

use mcb_domain::error::Result;
use mcb_domain::ports::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionId, CollectionInfo, FileInfo, SearchResult};

use super::QdrantVectorStoreProvider;

#[async_trait]
impl VectorStoreBrowser for QdrantVectorStoreProvider {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let response = self
            .request(reqwest::Method::GET, "/collections", None)
            .await?;

        let collections = response["result"]["collections"]
            .as_array()
            .ok_or_else(|| {
                mcb_domain::error::Error::vector_db(
                    "Qdrant list_collections: malformed response, missing collections array",
                )
            })?
            .iter()
            .map(|item| {
                let name = item["name"]
                    .as_str()
                    .ok_or_else(|| {
                        mcb_domain::error::Error::vector_db(
                            "Qdrant list_collections: missing collection name",
                        )
                    })?
                    .to_owned();
                Ok(CollectionInfo::new(name, 0, 0, None, self.provider_name()))
            })
            .collect::<Result<Vec<_>>>()?;

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
        let response = self
            .request_points_operation(
                reqwest::Method::POST,
                collection,
                "scroll",
                Some(serde_json::json!({
                    "filter": {
                        "must": [{
                            "key": crate::constants::VECTOR_FIELD_FILE_PATH,
                            "match": { "value": file_path }
                        }]
                    },
                    "limit": 100,
                    "with_payload": true
                })),
            )
            .await?;

        let mut results = Self::map_result_items(
            &response["result"]["points"],
            "payload missing or malformed, using empty default",
            "search_result.payload",
        )?;

        results.sort_by_key(|r| r.start_line);
        Ok(results)
    }
}
