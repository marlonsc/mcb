use super::*;

#[async_trait]
impl VectorStoreAdmin for EdgeVecVectorStoreProvider {
    async fn collection_exists(&self, collection: &CollectionId) -> Result<bool> {
        self.send_query(|tx| QueryMessage::CollectionExists {
            name: collection.to_string(),
            tx,
        })
        .await
    }

    async fn get_stats(
        &self,
        collection: &CollectionId,
    ) -> Result<HashMap<String, serde_json::Value>> {
        self.send_query(|tx| QueryMessage::GetStats {
            collection: collection.to_string(),
            tx,
        })
        .await
    }

    async fn flush(&self, _collection: &CollectionId) -> Result<()> {
        // EdgeVec uses synchronous in-memory writes — flush is a no-op
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "edgevec"
    }
}

#[async_trait]
impl VectorStoreBrowser for EdgeVecVectorStoreProvider {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        send_actor_msg!(self, Browse(BrowseMessage::ListCollections {}))
    }

    async fn list_file_paths(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<FileInfo>> {
        self.send_browse(|tx| BrowseMessage::ListFilePaths {
            collection: collection.to_string(),
            limit,
            tx,
        })
        .await
    }

    async fn get_chunks_by_file(
        &self,
        collection: &CollectionId,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        self.send_browse(|tx| BrowseMessage::GetChunksByFile {
            collection: collection.to_string(),
            file_path: file_path.to_owned(),
            tx,
        })
        .await
    }
}

#[async_trait]
impl VectorStoreProvider for EdgeVecVectorStoreProvider {
    async fn create_collection(&self, collection: &CollectionId, _dimensions: usize) -> Result<()> {
        self.send_core(|tx| CoreMessage::CreateCollection {
            name: collection.to_string(),
            tx,
        })
        .await
    }

    async fn delete_collection(&self, collection: &CollectionId) -> Result<()> {
        self.send_core(|tx| CoreMessage::DeleteCollection {
            name: collection.to_string(),
            tx,
        })
        .await
    }

    async fn insert_vectors(
        &self,
        collection: &CollectionId,
        vectors: &[Embedding],
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        self.send_core(|tx| CoreMessage::InsertVectors {
            collection: collection.to_string(),
            vectors: vectors.to_vec(),
            metadata,
            tx,
        })
        .await
    }

    async fn search_similar(
        &self,
        collection: &CollectionId,
        query_vector: &[f32],
        limit: usize,
        _filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        self.send_core(|tx| CoreMessage::SearchSimilar {
            collection: collection.to_string(),
            query_vector: query_vector.to_vec(),
            limit,
            tx,
        })
        .await
    }

    async fn delete_vectors(&self, collection: &CollectionId, ids: &[String]) -> Result<()> {
        self.send_core(|tx| CoreMessage::DeleteVectors {
            collection: collection.to_string(),
            ids: ids.to_vec(),
            tx,
        })
        .await
    }

    async fn get_vectors_by_ids(
        &self,
        collection: &CollectionId,
        ids: &[String],
    ) -> Result<Vec<SearchResult>> {
        self.send_query(|tx| QueryMessage::GetVectorsByIds {
            collection: collection.to_string(),
            ids: ids.to_vec(),
            tx,
        })
        .await
    }

    async fn list_vectors(
        &self,
        collection: &CollectionId,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        self.send_query(|tx| QueryMessage::ListVectors {
            collection: collection.to_string(),
            limit,
            tx,
        })
        .await
    }
}
