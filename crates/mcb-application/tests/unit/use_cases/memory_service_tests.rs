use std::sync::atomic::{AtomicU64, Ordering};

use mcb_application::use_cases::memory_service::MemoryServiceImpl;
use mcb_domain::entities::memory::{MemoryFilter, ObservationMetadata, ObservationType};
use mcb_domain::ports::MemoryServiceInterface;
use rstest::*;
use serial_test::serial;

use crate::utils::TEST_PROJECT_ID;
use crate::utils::shared_context::try_shared_app_context;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn test_epoch_secs_i64_reports_recent_time() {
    let ts = mcb_domain::utils::time::epoch_secs_i64().expect("clock ok");
    assert!(ts > 1_700_000_000, "Timestamp should be after 2023");
    assert!(ts < 2_000_000_000, "Timestamp should be before 2033");
}

#[fixture]
async fn memory_service() -> Option<MemoryServiceImpl> {
    let app_ctx = try_shared_app_context()?;
    let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    Some(MemoryServiceImpl::new(
        format!("test-project-{id}"),
        app_ctx.memory_repository(),
        app_ctx.embedding_handle().get(),
        app_ctx.vector_store_handle().get(),
    ))
}

mod integration_tests {
    use super::*;

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn test_hybrid_search_combines_fts_and_vector(
        #[future] memory_service: Option<MemoryServiceImpl>,
    ) {
        let Some(service) = memory_service.await else {
            eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
            return;
        };

        // Store observations
        let (id_a, _) = service
            .store_observation(
                TEST_PROJECT_ID.to_owned(),
                "content about rust generics and trait bounds".to_owned(),
                ObservationType::Context,
                vec![],
                ObservationMetadata::default(),
            )
            .await
            .expect("store obs a");

        let (id_b, _) = service
            .store_observation(
                TEST_PROJECT_ID.to_owned(),
                "content about python dynamic types".to_owned(),
                ObservationType::Context,
                vec![],
                ObservationMetadata::default(),
            )
            .await
            .expect("store obs b");

        let results = service
            .search_memories("rust generics", None, 10)
            .await
            .expect("search should succeed");

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
    #[serial]
    async fn test_search_respects_memory_filter(
        #[future] memory_service: Option<MemoryServiceImpl>,
    ) {
        let Some(service) = memory_service.await else {
            eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
            return;
        };

        let meta1 = ObservationMetadata {
            session_id: Some("session-1".to_owned()),
            ..Default::default()
        };
        let (id_a, _) = service
            .store_observation(
                TEST_PROJECT_ID.to_owned(),
                "session one observation".to_owned(),
                ObservationType::Context,
                vec![],
                meta1,
            )
            .await
            .expect("store a");

        let meta2 = ObservationMetadata {
            session_id: Some("session-2".to_owned()),
            ..Default::default()
        };
        let (_id_b, _) = service
            .store_observation(
                TEST_PROJECT_ID.to_owned(),
                "session two observation".to_owned(),
                ObservationType::Context,
                vec![],
                meta2,
            )
            .await
            .expect("store b");

        let filter = MemoryFilter {
            session_id: Some("session-1".to_owned()),
            ..Default::default()
        };

        let results = service
            .search_memories("observation", Some(filter), 10)
            .await
            .expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id_a.as_str());
    }

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn test_filter_by_branch(#[future] memory_service: Option<MemoryServiceImpl>) {
        let Some(service) = memory_service.await else {
            eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
            return;
        };

        let meta1 = ObservationMetadata {
            branch: Some("feature/auth".to_owned()),
            ..Default::default()
        };
        let (id_a, _) = service
            .store_observation(
                TEST_PROJECT_ID.to_owned(),
                "feature branch work".to_owned(),
                ObservationType::Context,
                vec![],
                meta1,
            )
            .await
            .expect("store a");

        let meta2 = ObservationMetadata {
            branch: Some("main".to_owned()),
            ..Default::default()
        };
        let (_id_b, _) = service
            .store_observation(
                TEST_PROJECT_ID.to_owned(),
                "main branch work".to_owned(),
                ObservationType::Context,
                vec![],
                meta2,
            )
            .await
            .expect("store b");

        let filter = MemoryFilter {
            branch: Some("feature/auth".to_owned()),
            ..Default::default()
        };

        let results = service
            .search_memories("work", Some(filter), 10)
            .await
            .expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id_a.as_str());
    }

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn test_filter_by_commit(#[future] memory_service: Option<MemoryServiceImpl>) {
        let Some(service) = memory_service.await else {
            eprintln!("skipping: shared AppContext unavailable (FastEmbed model missing)");
            return;
        };

        let meta1 = ObservationMetadata {
            commit: Some("abc1234".to_owned()),
            ..Default::default()
        };
        let (id_a, _) = service
            .store_observation(
                TEST_PROJECT_ID.to_owned(),
                "commit abc work".to_owned(),
                ObservationType::Context,
                vec![],
                meta1,
            )
            .await
            .expect("store a");

        let meta2 = ObservationMetadata {
            commit: Some("def5678".to_owned()),
            ..Default::default()
        };
        let (_id_b, _) = service
            .store_observation(
                TEST_PROJECT_ID.to_owned(),
                "commit def work".to_owned(),
                ObservationType::Context,
                vec![],
                meta2,
            )
            .await
            .expect("store b");

        let filter = MemoryFilter {
            commit: Some("abc1234".to_owned()),
            ..Default::default()
        };

        let results = service
            .search_memories("work", Some(filter), 10)
            .await
            .expect("search");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, id_a.as_str());
    }
}
