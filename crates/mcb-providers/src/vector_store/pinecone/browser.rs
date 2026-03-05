//! Browser operations for Pinecone vector store provider.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionId, CollectionInfo, FileInfo, SearchResult};

use mcb_utils::constants::vector_store::VECTOR_FIELD_FILE_PATH;

use super::PineconeVectorStoreProvider;

#[async_trait]
impl VectorStoreBrowser for PineconeVectorStoreProvider {
    // --- Browser Methods ---

    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        // Query Pinecone to get the authoritative list of namespaces.
        // `/describe_index_stats` returns all namespaces in the index.
        let response = self
            .request(
                reqwest::Method::POST,
                "/describe_index_stats",
                Some(serde_json::json!({ "filter": {} })),
            )
            .await?;

        let collections = response
            .get("namespaces")
            .and_then(serde_json::Value::as_object)
            .map(|namespaces| {
                namespaces
                    .keys()
                    .map(|name| {
                        let vector_count = namespaces
                            .get(name)
                            .and_then(|ns| ns.get("vectorCount"))
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(0);
                        CollectionInfo::new(
                            name,
                            vector_count as usize,
                            0,
                            None,
                            self.provider_name(),
                        )
                    })
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
        let filter = serde_json::json!({
            (VECTOR_FIELD_FILE_PATH): { "$eq": file_path }
        });

        let collection_str = collection.to_string();
        let dimensions = self.collection_dimensions(&collection_str);

        let zero_vector = vec![0.0f32; dimensions];

        let mut results = self
            .query_match_results(serde_json::json!({
                "vector": zero_vector,
                "topK": mcb_utils::constants::BROWSE_MAX_CHUNKS_PER_FILE,
                "namespace": collection_str,
                "includeMetadata": true,
                "filter": filter
            }))
            .await?;

        results.sort_by_key(|r| r.start_line);
        Ok(results)
    }
}
