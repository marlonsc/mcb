use mcb_application::use_cases::memory_service::MemoryServiceImpl;
use mcb_domain::entities::memory::{MemoryFilter, ObservationMetadata, ObservationType};
use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use rstest::*;

#[test]
fn test_current_timestamp_reports_recent_time() {
    let ts = MemoryServiceImpl::current_timestamp();
    assert!(ts > 1_700_000_000, "Timestamp should be after 2023");
    assert!(ts < 2_000_000_000, "Timestamp should be before 2033");
}

struct TestContext {
    service: MemoryServiceImpl,
    _temp: tempfile::TempDir,
}

#[fixture]
async fn ctx() -> TestContext {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(temp_dir.path().join("test.db"));

    let app_ctx = init_app(config).await.expect("init app context");
    let service = MemoryServiceImpl::new(
        "test-project".to_string(),
        app_ctx.memory_repository(),
        app_ctx.embedding_handle().get(),
        app_ctx.vector_store_handle().get(),
    );

    TestContext {
        service,
        _temp: temp_dir,
    }
}

mod integration_tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[tokio::test]
    async fn test_hybrid_search_combines_fts_and_vector(#[future] ctx: TestContext) {
        let ctx = ctx.await;

        // Store observations
        // 1. "rust generics" -> relevant to "generics"
        let (id_a, _) = ctx
            .service
            .store_observation(
                "test-project".to_string(),
                "content about rust generics and trait bounds".to_string(),
                ObservationType::Context,
                vec![],
                ObservationMetadata::default(),
            )
            .await
            .expect("store obs a");

        // 2. "python types" -> relevant to "types"
        let (id_b, _) = ctx
            .service
            .store_observation(
                "test-project".to_string(),
                "content about python dynamic types".to_string(),
                ObservationType::Context,
                vec![],
                ObservationMetadata::default(),
            )
            .await
            .expect("store obs b");

        // Search for "rust generics"
        let results = ctx
            .service
            .search_memories("rust generics", None, 10)
            .await
            .expect("search should succeed");

        // Should find id_a (high relevance) and possibly id_b (low relevance or none)
        // With hybrid search, exact matches in FTS or high similarity in Vector should rank higher.

        let found_a = results.iter().find(|r| r.id == id_a.as_str());
        assert!(found_a.is_some(), "Should find rust observation");

        // Ensure ranking makes sense if both returned
        if let Some(found_b) = results.iter().find(|r| r.id == id_b.as_str()) {
            let score_a = found_a.unwrap().similarity_score;
            let score_b = found_b.similarity_score;
            assert!(
                score_a > score_b,
                "Rust observation should be more relevant than Python one for 'rust generics' query"
            );
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_search_respects_memory_filter(#[future] ctx: TestContext) {
        let ctx = ctx.await;

        let meta1 = ObservationMetadata {
            session_id: Some("session-1".to_string()),
            ..Default::default()
        };
        let (id_a, _) = ctx
            .service
            .store_observation(
                "test-project".to_string(),
                "session one observation".to_string(),
                ObservationType::Context,
                vec![],
                meta1,
            )
            .await
            .expect("store a");

        let meta2 = ObservationMetadata {
            session_id: Some("session-2".to_string()),
            ..Default::default()
        };
        let (_id_b, _) = ctx
            .service
            .store_observation(
                "test-project".to_string(),
                "session two observation".to_string(),
                ObservationType::Context,
                vec![],
                meta2,
            )
            .await
            .expect("store b");

        let filter = MemoryFilter {
            session_id: Some("session-1".to_string()),
            ..Default::default()
        };

        let results = ctx
            .service
            .search_memories("observation", Some(filter), 10)
            .await
            .expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id_a.as_str());
    }

    #[rstest]
    #[tokio::test]
    async fn test_filter_by_branch(#[future] ctx: TestContext) {
        let ctx = ctx.await;

        let meta1 = ObservationMetadata {
            branch: Some("feature/auth".to_string()),
            ..Default::default()
        };
        let (id_a, _) = ctx
            .service
            .store_observation(
                "test-project".to_string(),
                "feature branch work".to_string(),
                ObservationType::Context,
                vec![],
                meta1,
            )
            .await
            .expect("store a");

        let meta2 = ObservationMetadata {
            branch: Some("main".to_string()),
            ..Default::default()
        };
        let (_id_b, _) = ctx
            .service
            .store_observation(
                "test-project".to_string(),
                "main branch work".to_string(),
                ObservationType::Context,
                vec![],
                meta2,
            )
            .await
            .expect("store b");

        let filter = MemoryFilter {
            branch: Some("feature/auth".to_string()),
            ..Default::default()
        };

        let results = ctx
            .service
            .search_memories("work", Some(filter), 10)
            .await
            .expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id_a.as_str());
    }

    #[rstest]
    #[tokio::test]
    async fn test_filter_by_commit(#[future] ctx: TestContext) {
        let ctx = ctx.await;

        let meta1 = ObservationMetadata {
            commit: Some("abc1234".to_string()),
            ..Default::default()
        };
        let (id_a, _) = ctx
            .service
            .store_observation(
                "test-project".to_string(),
                "commit abc work".to_string(),
                ObservationType::Context,
                vec![],
                meta1,
            )
            .await
            .expect("store a");

        let meta2 = ObservationMetadata {
            commit: Some("def5678".to_string()),
            ..Default::default()
        };
        let (_id_b, _) = ctx
            .service
            .store_observation(
                "test-project".to_string(),
                "commit def work".to_string(),
                ObservationType::Context,
                vec![],
                meta2,
            )
            .await
            .expect("store b");

        let filter = MemoryFilter {
            commit: Some("abc1234".to_string()),
            ..Default::default()
        };

        let results = ctx
            .service
            .search_memories("work", Some(filter), 10)
            .await
            .expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id_a.as_str());
    }
}
