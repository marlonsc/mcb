use mcb_application::use_cases::memory_service::MemoryServiceImpl;

#[test]
fn test_current_timestamp_reports_recent_time() {
    let ts = MemoryServiceImpl::current_timestamp();
    assert!(ts > 1_700_000_000, "Timestamp should be after 2023");
    assert!(ts < 2_000_000_000, "Timestamp should be before 2033");
}

#[cfg(test)]
mod rrf_tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use mcb_application::use_cases::memory_service::MemoryServiceImpl;
    use mcb_domain::entities::memory::{
        MemoryFilter, MemorySearchResult, Observation, ObservationMetadata, ObservationType,
        SessionSummary,
    };
    use mcb_domain::error::Result;
    use mcb_domain::ports::providers::EmbeddingProvider;
    use mcb_domain::ports::providers::vector_store::VectorStoreProvider;
    use mcb_domain::ports::providers::vector_store::{VectorStoreAdmin, VectorStoreBrowser};
    use mcb_domain::ports::repositories::memory_repository::{FtsSearchResult, MemoryRepository};
    use mcb_domain::ports::services::MemoryServiceInterface;
    use mcb_domain::utils::compute_content_hash;
    use mcb_domain::value_objects::{
        CollectionId, Embedding, ObservationId, SearchResult, SessionId,
    };
    use serde_json::Value;

    // ---- Mock EmbeddingProvider ----

    struct MockEmbedding;

    #[async_trait]
    impl EmbeddingProvider for MockEmbedding {
        async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Embedding>> {
            Ok(texts
                .iter()
                .map(|_| Embedding {
                    vector: vec![0.1, 0.2, 0.3],
                    model: "mock".to_string(),
                    dimensions: 3,
                })
                .collect())
        }

        fn dimensions(&self) -> usize {
            3
        }

        fn provider_name(&self) -> &str {
            "mock"
        }
    }

    // ---- Mock VectorStoreProvider ----

    struct MockVectorStore {
        results: Vec<SearchResult>,
    }

    #[async_trait]
    impl VectorStoreAdmin for MockVectorStore {
        async fn collection_exists(&self, _name: &CollectionId) -> Result<bool> {
            Ok(true)
        }

        async fn get_stats(&self, _collection: &CollectionId) -> Result<HashMap<String, Value>> {
            Ok(HashMap::new())
        }

        async fn flush(&self, _collection: &CollectionId) -> Result<()> {
            Ok(())
        }

        fn provider_name(&self) -> &str {
            "mock"
        }
    }

    #[async_trait]
    impl VectorStoreBrowser for MockVectorStore {
        async fn list_collections(&self) -> Result<Vec<mcb_domain::value_objects::CollectionInfo>> {
            Ok(vec![])
        }

        async fn list_file_paths(
            &self,
            _collection: &CollectionId,
            _limit: usize,
        ) -> Result<Vec<mcb_domain::value_objects::FileInfo>> {
            Ok(vec![])
        }

        async fn get_chunks_by_file(
            &self,
            _collection: &CollectionId,
            _file_path: &str,
        ) -> Result<Vec<SearchResult>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl VectorStoreProvider for MockVectorStore {
        async fn create_collection(&self, _name: &CollectionId, _dimensions: usize) -> Result<()> {
            Ok(())
        }

        async fn delete_collection(&self, _name: &CollectionId) -> Result<()> {
            Ok(())
        }

        async fn insert_vectors(
            &self,
            _collection: &CollectionId,
            _vectors: &[Embedding],
            _metadata: Vec<HashMap<String, Value>>,
        ) -> Result<Vec<String>> {
            Ok(vec!["mock-id".to_string()])
        }

        async fn search_similar(
            &self,
            _collection: &CollectionId,
            _query_vector: &[f32],
            _limit: usize,
            _filter: Option<&str>,
        ) -> Result<Vec<SearchResult>> {
            Ok(self.results.clone())
        }

        async fn delete_vectors(&self, _collection: &CollectionId, _ids: &[String]) -> Result<()> {
            Ok(())
        }

        async fn get_vectors_by_ids(
            &self,
            _collection: &CollectionId,
            _ids: &[String],
        ) -> Result<Vec<SearchResult>> {
            Ok(vec![])
        }

        async fn list_vectors(
            &self,
            _collection: &CollectionId,
            _limit: usize,
        ) -> Result<Vec<SearchResult>> {
            Ok(vec![])
        }
    }

    // ---- Mock MemoryRepository ----

    struct MockMemoryRepo {
        observations: Vec<Observation>,
        fts_results: Vec<FtsSearchResult>,
    }

    #[async_trait]
    impl MemoryRepository for MockMemoryRepo {
        async fn store_observation(&self, _observation: &Observation) -> Result<()> {
            Ok(())
        }

        async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>> {
            Ok(self
                .observations
                .iter()
                .find(|o| o.id == id.as_str())
                .cloned())
        }

        async fn find_by_hash(&self, content_hash: &str) -> Result<Option<Observation>> {
            Ok(self
                .observations
                .iter()
                .find(|o| o.content_hash == content_hash)
                .cloned())
        }

        async fn search_fts(&self, _query: &str, _limit: usize) -> Result<Vec<String>> {
            Ok(self.fts_results.iter().map(|r| r.id.clone()).collect())
        }

        async fn search_fts_ranked(
            &self,
            _query: &str,
            _limit: usize,
        ) -> Result<Vec<FtsSearchResult>> {
            Ok(self.fts_results.clone())
        }

        async fn delete_observation(&self, _id: &ObservationId) -> Result<()> {
            Ok(())
        }

        async fn search(
            &self,
            _query_embedding: &[f32],
            _filter: MemoryFilter,
            _limit: usize,
        ) -> Result<Vec<MemorySearchResult>> {
            Ok(vec![])
        }

        async fn get_observations_by_ids(&self, ids: &[ObservationId]) -> Result<Vec<Observation>> {
            let id_strings: Vec<&str> = ids.iter().map(|id| id.as_str()).collect();
            Ok(self
                .observations
                .iter()
                .filter(|o| id_strings.contains(&o.id.as_str()))
                .cloned()
                .collect())
        }

        async fn get_timeline(
            &self,
            _anchor_id: &ObservationId,
            _before: usize,
            _after: usize,
            _filter: Option<MemoryFilter>,
        ) -> Result<Vec<Observation>> {
            Ok(vec![])
        }

        async fn store_session_summary(&self, _summary: &SessionSummary) -> Result<()> {
            Ok(())
        }

        async fn get_session_summary(
            &self,
            _session_id: &SessionId,
        ) -> Result<Option<SessionSummary>> {
            Ok(None)
        }
    }

    // ---- Helper ----

    fn make_observation(id: &str, content: &str) -> Observation {
        Observation {
            id: id.to_string(),
            project_id: "test-project".to_string(),
            content: content.to_string(),
            content_hash: compute_content_hash(content),
            tags: vec![],
            observation_type: ObservationType::Context,
            metadata: ObservationMetadata::default(),
            created_at: 1_700_000_000,
            embedding_id: None,
        }
    }

    // ---- Tests ----

    /// Verifies Reciprocal Rank Fusion correctly combines FTS and vector results.
    #[tokio::test]
    async fn test_rrf_hybrid_search_combines_fts_and_vector() {
        let obs_a = make_observation("obs-a", "content about rust generics");
        let obs_b = make_observation("obs-b", "content about python types");

        let fts_results = vec![
            FtsSearchResult {
                id: "obs-b".to_string(),
                rank: -2.0,
            },
            FtsSearchResult {
                id: "obs-a".to_string(),
                rank: -1.5,
            },
        ];

        let vector_results = vec![SearchResult {
            id: "vec-1".to_string(),
            file_path: String::new(),
            start_line: 0,
            content: "content about rust generics".to_string(),
            score: 0.95,
            language: "rust".to_string(),
        }];

        let repo = Arc::new(MockMemoryRepo {
            observations: vec![obs_a.clone(), obs_b.clone()],
            fts_results,
        });

        let vector_store = Arc::new(MockVectorStore {
            results: vector_results,
        });

        let embedding_provider = Arc::new(MockEmbedding);

        let service = MemoryServiceImpl::new(
            "test-project".to_string(),
            repo,
            embedding_provider,
            vector_store,
        );

        let results = service
            .search_memories("rust generics", None, 10)
            .await
            .expect("search should succeed");

        assert!(results.len() >= 2);
        assert_eq!(results[0].id, "obs-a");
        assert_eq!(results[1].id, "obs-b");
        assert!(results[0].similarity_score > results[1].similarity_score);
    }

    #[tokio::test]
    async fn test_rrf_fallback_to_fts_when_vector_empty() {
        let obs_a = make_observation("obs-a", "debugging tokio runtime");
        let obs_b = make_observation("obs-b", "async runtime patterns");

        let fts_results = vec![
            FtsSearchResult {
                id: "obs-a".to_string(),
                rank: -2.5,
            },
            FtsSearchResult {
                id: "obs-b".to_string(),
                rank: -1.0,
            },
        ];

        let repo = Arc::new(MockMemoryRepo {
            observations: vec![obs_a, obs_b],
            fts_results,
        });

        let vector_store = Arc::new(MockVectorStore { results: vec![] });
        let embedding_provider = Arc::new(MockEmbedding);

        let service = MemoryServiceImpl::new(
            "test-project".to_string(),
            repo,
            embedding_provider,
            vector_store,
        );

        let results = service
            .search_memories("tokio runtime", None, 10)
            .await
            .expect("search should succeed");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "obs-a");
        assert_eq!(results[1].id, "obs-b");
    }

    #[tokio::test]
    async fn test_rrf_respects_memory_filter() {
        let mut obs_a = make_observation("obs-a", "session one observation");
        obs_a.metadata.session_id = Some("session-1".to_string());

        let mut obs_b = make_observation("obs-b", "session two observation");
        obs_b.metadata.session_id = Some("session-2".to_string());

        let fts_results = vec![
            FtsSearchResult {
                id: "obs-a".to_string(),
                rank: -2.0,
            },
            FtsSearchResult {
                id: "obs-b".to_string(),
                rank: -1.5,
            },
        ];

        let repo = Arc::new(MockMemoryRepo {
            observations: vec![obs_a, obs_b],
            fts_results,
        });

        let vector_store = Arc::new(MockVectorStore { results: vec![] });
        let embedding_provider = Arc::new(MockEmbedding);

        let service = MemoryServiceImpl::new(
            "test-project".to_string(),
            repo,
            embedding_provider,
            vector_store,
        );

        let filter = MemoryFilter {
            session_id: Some("session-1".to_string()),
            ..Default::default()
        };

        let results = service
            .search_memories("observation", Some(filter), 10)
            .await
            .expect("search should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "obs-a");
    }

    #[tokio::test]
    async fn test_filter_by_branch() {
        let mut obs_a = make_observation("obs-a", "feature branch work");
        obs_a.metadata.branch = Some("feature/auth".to_string());

        let mut obs_b = make_observation("obs-b", "main branch work");
        obs_b.metadata.branch = Some("main".to_string());

        let fts_results = vec![
            FtsSearchResult {
                id: "obs-a".to_string(),
                rank: -2.0,
            },
            FtsSearchResult {
                id: "obs-b".to_string(),
                rank: -1.5,
            },
        ];

        let repo = Arc::new(MockMemoryRepo {
            observations: vec![obs_a, obs_b],
            fts_results,
        });

        let vector_store = Arc::new(MockVectorStore { results: vec![] });
        let embedding_provider = Arc::new(MockEmbedding);

        let service = MemoryServiceImpl::new(
            "test-project".to_string(),
            repo,
            embedding_provider,
            vector_store,
        );

        let filter = MemoryFilter {
            branch: Some("feature/auth".to_string()),
            ..Default::default()
        };

        let results = service
            .search_memories("branch work", Some(filter), 10)
            .await
            .expect("search should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "obs-a");
    }

    #[tokio::test]
    async fn test_filter_by_commit() {
        let mut obs_a = make_observation("obs-a", "commit abc observation");
        obs_a.metadata.commit = Some("abc123".to_string());

        let mut obs_b = make_observation("obs-b", "commit def observation");
        obs_b.metadata.commit = Some("def456".to_string());

        let fts_results = vec![
            FtsSearchResult {
                id: "obs-a".to_string(),
                rank: -2.0,
            },
            FtsSearchResult {
                id: "obs-b".to_string(),
                rank: -1.5,
            },
        ];

        let repo = Arc::new(MockMemoryRepo {
            observations: vec![obs_a, obs_b],
            fts_results,
        });

        let vector_store = Arc::new(MockVectorStore { results: vec![] });
        let embedding_provider = Arc::new(MockEmbedding);

        let service = MemoryServiceImpl::new(
            "test-project".to_string(),
            repo,
            embedding_provider,
            vector_store,
        );

        let filter = MemoryFilter {
            commit: Some("abc123".to_string()),
            ..Default::default()
        };

        let results = service
            .search_memories("commit observation", Some(filter), 10)
            .await
            .expect("search should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "obs-a");
    }
}
