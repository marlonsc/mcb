//! Browser operations for Pinecone vector store provider.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::{VectorStoreBrowser, VectorStoreAdmin, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionId, CollectionInfo, FileInfo, SearchResult};

use crate::constants::VECTOR_FIELD_FILE_PATH;

use super::PineconeVectorStoreProvider;

#[async_trait]
impl VectorStoreBrowser for PineconeVectorStoreProvider {
    // --- Browser Methods ---

    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        let collections: Vec<CollectionInfo> = self
            .collections
            .iter()
            .map(|entry| CollectionInfo::new(entry.key(), 0, 0, None, self.provider_name()))
            .collect();
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
                "topK": 100,
                "namespace": collection_str,
                "includeMetadata": true,
                "filter": filter
            }))
            .await?;

        results.sort_by_key(|r| r.start_line);
        Ok(results)
    }
}
