use super::*;

pub struct EdgeVecActor {
    receiver: mpsc::Receiver<EdgeVecMessage>,
    index: edgevec::HnswIndex,
    storage: edgevec::VectorStorage,
    metadata_store: DashMap<String, CollectionMetadata>,
    id_map: DashMap<String, VectorId>,
    config: EdgeVecConfig,
}

impl EdgeVecActor {
    pub fn new(receiver: mpsc::Receiver<EdgeVecMessage>, config: EdgeVecConfig) -> Result<Self> {
        let hnsw_config = edgevec::HnswConfig {
            m: config.hnsw_config.m,
            m0: config.hnsw_config.m0,
            ef_construction: config.hnsw_config.ef_construction,
            ef_search: config.hnsw_config.ef_search,
            dimensions: config.dimensions as u32,
            metric: match config.metric {
                MetricType::L2Squared => edgevec::HnswConfig::METRIC_L2_SQUARED,
                MetricType::Cosine => edgevec::HnswConfig::METRIC_COSINE,
                MetricType::DotProduct => edgevec::HnswConfig::METRIC_DOT_PRODUCT,
            },
            _reserved: [0; 2],
        };

        let storage = edgevec::VectorStorage::new(&hnsw_config, None);
        let index = edgevec::HnswIndex::new(hnsw_config, &storage)
            .map_err(|e| Error::vector_db(format!("Failed to create EdgeVec HNSW index: {e}")))?;

        Ok(Self {
            receiver,
            index,
            storage,
            metadata_store: DashMap::new(),
            id_map: DashMap::new(),
            config,
        })
    }
}

impl EdgeVecActor {
    fn handle_create_collection(&self, name: String) -> Result<()> {
        self.metadata_store.insert(name, HashMap::new());
        Ok(())
    }

    fn handle_delete_collection(&mut self, name: &str) -> Result<()> {
        if let Some((_, collection_metadata)) = self.metadata_store.remove(name) {
            for external_id in collection_metadata.keys() {
                if let Some(vector_id) = self.id_map.remove(external_id) {
                    let _ = self.index.soft_delete(vector_id.1);
                }
            }
        }
        Ok(())
    }

    fn handle_collection_exists(&self, name: &str) -> Result<bool> {
        Ok(self.metadata_store.contains_key(name))
    }
}

impl EdgeVecActor {
    fn get_collection_metadata(
        &self,
        name: &str,
    ) -> Option<dashmap::mapref::one::Ref<'_, String, CollectionMetadata>> {
        self.metadata_store.get(name)
    }

    fn collection_metadata_len(&self, name: &str) -> usize {
        self.get_collection_metadata(name).map_or(0, |m| m.len())
    }

    fn handle_insert_vectors(
        &mut self,
        collection: &str,
        vectors: Vec<Embedding>,
        metadata: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<String>> {
        let mut ids = Vec::with_capacity(vectors.len());
        let mut collection_metadata = self
            .metadata_store
            .entry(collection.to_owned())
            .or_default();

        for (vector, meta) in vectors.into_iter().zip(metadata.into_iter()) {
            let external_id = format!("{}_{}", collection, id::generate());

            match self.index.insert(&vector.vector, &mut self.storage) {
                Ok(vector_id) => {
                    self.id_map.insert(external_id.clone(), vector_id);
                    let mut enriched_metadata = meta.clone();
                    enriched_metadata.insert("id".to_owned(), serde_json::json!(external_id));
                    collection_metadata
                        .insert(external_id.clone(), serde_json::json!(enriched_metadata));
                    ids.push(external_id);
                }
                Err(e) => {
                    return Err(Error::vector_db(format!("Failed to insert vector: {e}")));
                }
            }
        }
        Ok(ids)
    }

    fn handle_delete_vectors(&mut self, collection: &str, ids: Vec<String>) -> Result<()> {
        if let Some(mut collection_metadata) = self.metadata_store.get_mut(collection) {
            for id in ids {
                if let Some((_, vector_id)) = self.id_map.remove(&id) {
                    let _ = self.index.soft_delete(vector_id);
                }
                collection_metadata.remove(&id);
            }
        }
        Ok(())
    }

    fn handle_get_vectors_by_ids(&self, collection: &str, ids: Vec<String>) -> Vec<SearchResult> {
        let mut final_results = Vec::new();
        if let Some(collection_metadata) = self.get_collection_metadata(collection) {
            for id in ids {
                if let Some(meta_val) = collection_metadata.get(&id) {
                    final_results.push(search_result_from_json_metadata(id.clone(), meta_val, 1.0));
                }
            }
        }
        final_results
    }

    fn handle_list_vectors(&self, collection: &str, limit: usize) -> Vec<SearchResult> {
        let mut final_results = Vec::new();
        if let Some(collection_metadata) = self.get_collection_metadata(collection) {
            for (ext_id, meta_val) in collection_metadata.iter().take(limit) {
                final_results.push(search_result_from_json_metadata(
                    ext_id.clone(),
                    meta_val,
                    1.0,
                ));
            }
        }
        final_results
    }
}

impl EdgeVecActor {
    fn handle_search_similar(
        &self,
        collection: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // The HNSW index is global (shared across all collections), so when
        // multiple collections exist we must over-fetch to ensure enough
        // results survive the per-collection metadata filter.
        let total_vectors = self.index.len();
        let collection_size = self.collection_metadata_len(collection);
        let fetch_limit = if collection_size > 0 && total_vectors > collection_size {
            let ratio = (total_vectors as f64 / collection_size as f64).ceil() as usize;
            (limit * ratio).min(total_vectors)
        } else {
            limit
        };

        match self.index.search(query_vector, fetch_limit, &self.storage) {
            Ok(results) => {
                let mut final_results = Vec::with_capacity(limit);
                if let Some(collection_metadata) = self.get_collection_metadata(collection) {
                    for res in results {
                        if final_results.len() >= limit {
                            break;
                        }
                        let external_id: Option<String> = self
                            .id_map
                            .iter()
                            .find(|entry| *entry.value() == res.vector_id)
                            .map(|entry| entry.key().to_owned());

                        if let Some(ext_id) = external_id
                            && let Some(meta_val) = collection_metadata.get(&ext_id)
                        {
                            final_results.push(search_result_from_json_metadata(
                                ext_id,
                                meta_val,
                                res.distance as f64,
                            ));
                        }
                    }
                }
                Ok(final_results)
            }
            Err(e) => Err(Error::vector_db(format!("Search failed: {e}"))),
        }
    }
}

impl EdgeVecActor {
    fn handle_get_stats(&self, collection: &str) -> HashMap<String, serde_json::Value> {
        let vector_count = self.collection_metadata_len(collection);
        let mut stats = HashMap::new();
        stats.insert(
            STATS_FIELD_COLLECTION.to_owned(),
            serde_json::json!(collection),
        );
        stats.insert(
            STATS_FIELD_VECTORS_COUNT.to_owned(),
            serde_json::json!(vector_count),
        );
        stats.insert(
            "total_indexed_vectors".to_owned(),
            serde_json::json!(self.index.len()),
        );
        stats.insert(
            "dimensions".to_owned(),
            serde_json::json!(self.config.dimensions),
        );
        stats
    }
}

impl EdgeVecActor {
    fn handle_list_collections(&self) -> Vec<CollectionInfo> {
        self.metadata_store
            .iter()
            .map(|entry| {
                let name = entry.key().clone();
                let vector_count = entry.value().len() as u64;

                // Count unique file paths
                let file_paths: std::collections::HashSet<&str> = entry
                    .value()
                    .values()
                    .filter_map(|v| {
                        v.as_object()
                            .and_then(|o| o.get(VECTOR_FIELD_FILE_PATH))
                            .and_then(|v| v.as_str())
                    })
                    .collect();
                let file_count = file_paths.len() as u64;

                CollectionInfo::new(name, vector_count, file_count, None, "edgevec")
            })
            .collect()
    }

    fn handle_list_file_paths(&self, collection: &str, limit: usize) -> Result<Vec<FileInfo>> {
        let collection_metadata = self
            .get_collection_metadata(collection)
            .ok_or_else(|| Error::vector_db(format!("Collection '{collection}' not found")))?;

        let mut file_map: HashMap<String, (u32, String)> = HashMap::new();

        for meta_val in collection_metadata.values() {
            if let Some(meta) = meta_val.as_object()
                && let Some(file_path) = meta.get(VECTOR_FIELD_FILE_PATH).and_then(|v| v.as_str())
            {
                let language = meta
                    .get(VECTOR_FIELD_LANGUAGE)
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_owned();

                let entry = file_map
                    .entry(file_path.to_owned())
                    .or_insert((0, language));
                entry.0 += 1;
            }
        }

        let files = file_map
            .into_iter()
            .take(limit)
            .map(|(path, (chunk_count, language))| FileInfo::new(path, chunk_count, language, None))
            .collect();
        Ok(files)
    }

    fn handle_get_chunks_by_file(
        &self,
        collection: &str,
        file_path: &str,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        // Normalize to forward slashes for cross-platform path matching
        let normalized_query = file_path.replace('\\', "/");
        if let Some(collection_metadata) = self.get_collection_metadata(collection) {
            for (ext_id, meta_val) in collection_metadata.iter() {
                if let Some(meta) = meta_val.as_object()
                    && meta
                        .get(VECTOR_FIELD_FILE_PATH)
                        .and_then(|v| v.as_str())
                        .is_some_and(|p| p.replace('\\', "/") == normalized_query)
                {
                    let mut result =
                        search_result_from_json_metadata(ext_id.clone(), meta_val, 1.0);
                    result.file_path = file_path.to_owned();
                    results.push(result);
                }
            }
        }
        // Sort by start_line
        results.sort_by_key(|r| r.start_line);
        Ok(results)
    }
}

impl EdgeVecActor {
    pub async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                EdgeVecMessage::Core(core) => self.handle_core_message(core),
                EdgeVecMessage::Query(query) => self.handle_query_message(query),
                EdgeVecMessage::Browse(browse) => self.handle_browse_message(browse),
            }
        }
    }

    fn handle_core_message(&mut self, msg: CoreMessage) {
        match msg {
            CoreMessage::CreateCollection { name, tx } => {
                let _ = tx.send(self.handle_create_collection(name));
            }
            CoreMessage::DeleteCollection { name, tx } => {
                let _ = tx.send(self.handle_delete_collection(&name));
            }
            CoreMessage::InsertVectors {
                collection,
                vectors,
                metadata,
                tx,
            } => {
                let _ = tx.send(self.handle_insert_vectors(&collection, vectors, metadata));
            }
            CoreMessage::SearchSimilar {
                collection,
                query_vector,
                limit,
                tx,
            } => {
                let _ = tx.send(self.handle_search_similar(&collection, &query_vector, limit));
            }
            CoreMessage::DeleteVectors {
                collection,
                ids,
                tx,
            } => {
                let _ = tx.send(self.handle_delete_vectors(&collection, ids));
            }
        }
    }

    fn handle_query_message(&mut self, msg: QueryMessage) {
        match msg {
            QueryMessage::GetStats { collection, tx } => {
                let _ = tx.send(Ok(self.handle_get_stats(&collection)));
            }
            QueryMessage::ListVectors {
                collection,
                limit,
                tx,
            } => {
                let _ = tx.send(Ok(self.handle_list_vectors(&collection, limit)));
            }
            QueryMessage::GetVectorsByIds {
                collection,
                ids,
                tx,
            } => {
                let _ = tx.send(Ok(self.handle_get_vectors_by_ids(&collection, ids)));
            }
            QueryMessage::CollectionExists { name, tx } => {
                let _ = tx.send(self.handle_collection_exists(&name));
            }
        }
    }

    fn handle_browse_message(&mut self, msg: BrowseMessage) {
        match msg {
            BrowseMessage::ListCollections { tx } => {
                let _ = tx.send(Ok(self.handle_list_collections()));
            }
            BrowseMessage::ListFilePaths {
                collection,
                limit,
                tx,
            } => {
                let _ = tx.send(self.handle_list_file_paths(&collection, limit));
            }
            BrowseMessage::GetChunksByFile {
                collection,
                file_path,
                tx,
            } => {
                let _ = tx.send(self.handle_get_chunks_by_file(&collection, &file_path));
            }
        }
    }
}
