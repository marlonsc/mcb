//! Observation repository integration tests.
use mcb_domain::entities::memory::{
    MemoryFilter, Observation, ObservationMetadata, ObservationType, SessionSummary,
};
use mcb_domain::ports::MemoryRepository;
use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::value_objects::{ObservationId, SessionId};
use mcb_providers::database::seaorm::repos::SeaOrmObservationRepository;
use mcb_utils::constants::values::DEFAULT_ORG_ID;
use mcb_utils::utils::id::compute_content_hash;
use rstest::rstest;
use sea_orm::{Database, DatabaseConnection};

fn make_observation(
    id: &str,
    org_id: &str,
    content: &str,
    tags: &[&str],
    created_at: i64,
    session_id: &str,
) -> Observation {
    Observation {
        id: id.to_owned(),
        project_id: format!("proj-memory-{org_id}"),
        org_id: org_id.to_owned(),
        content: content.to_owned(),
        content_hash: compute_content_hash(content),
        tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
        r#type: ObservationType::Context,
        metadata: ObservationMetadata {
            id: format!("meta-{id}"),
            session_id: Some(session_id.to_owned()),
            repo_id: Some("repo-1".to_owned()),
            file_path: None,
            branch: Some("main".to_owned()),
            commit: Some("abc123".to_owned()),
            execution: None,
            quality_gate: None,
            origin_context: None,
        },
        created_at,
        embedding_id: None,
    }
}

async fn setup_repo() -> TestResult<SeaOrmObservationRepository> {
    let db: DatabaseConnection = Database::connect(mcb_utils::constants::SQLITE_MEMORY_DSN).await?;

    mcb_domain::registry::database::migrate_up(Box::new(db.clone()), None).await?;

    Ok(SeaOrmObservationRepository::new(db))
}

#[rstest]
#[tokio::test]
async fn observation_repo_round_trip_store_get_list_timeline_and_inject() -> TestResult {
    let repo = setup_repo().await?;

    let obs1 = make_observation(
        "11111111-1111-1111-1111-111111111111",
        DEFAULT_ORG_ID,
        "first observation content",
        &["important", "alpha"],
        1_700_000_001,
        "ses-1",
    );
    let obs2 = make_observation(
        "22222222-2222-2222-2222-222222222222",
        DEFAULT_ORG_ID,
        "second observation content",
        &["important", "beta"],
        1_700_000_002,
        "ses-1",
    );
    let obs3 = make_observation(
        "33333333-3333-3333-3333-333333333333",
        DEFAULT_ORG_ID,
        "third observation content",
        &["gamma"],
        1_700_000_003,
        "ses-2",
    );

    repo.store_observation(&obs1).await?;
    repo.store_observation(&obs2).await?;
    repo.store_observation(&obs3).await?;

    let fetched = repo
        .get_observation(
            DEFAULT_ORG_ID,
            &ObservationId::from_string("22222222-2222-2222-2222-222222222222"),
        )
        .await?
        .ok_or("obs2 should exist")?;
    assert_eq!(fetched.id, "22222222-2222-2222-2222-222222222222");
    assert_eq!(fetched.content, "second observation content");

    let filtered = repo
        .list_observations(
            DEFAULT_ORG_ID,
            Some(&MemoryFilter {
                tags: Some(vec!["important".to_owned()]),
                session_id: Some("ses-1".to_owned()),
                ..Default::default()
            }),
            10,
        )
        .await?;

    let filtered_ids: Vec<&str> = filtered.iter().map(|obs| obs.id.as_str()).collect();
    assert_eq!(
        filtered_ids,
        vec![
            "22222222-2222-2222-2222-222222222222",
            "11111111-1111-1111-1111-111111111111",
        ]
    );

    let anchor_id = ObservationId::from_string("22222222-2222-2222-2222-222222222222");
    let timeline = repo
        .get_timeline(mcb_domain::ports::TimelineQuery {
            org_id: DEFAULT_ORG_ID,
            anchor_id: &anchor_id,
            before: 1,
            after: 1,
            filter: None,
        })
        .await?;
    let timeline_ids: Vec<&str> = timeline.iter().map(|obs| obs.id.as_str()).collect();
    assert_eq!(
        timeline_ids,
        vec![
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
            "33333333-3333-3333-3333-333333333333",
        ]
    );

    let injected = repo
        .inject_observations(
            DEFAULT_ORG_ID,
            Some(&MemoryFilter {
                tags: Some(vec!["important".to_owned()]),
                ..Default::default()
            }),
            10,
            128,
        )
        .await?;
    assert!(!injected.is_empty());
    assert!(
        injected
            .iter()
            .all(|obs| obs.tags.iter().any(|tag| tag == "important"))
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn search_handles_empty_query_for_memory_list_bug_regression() -> TestResult {
    let repo = setup_repo().await?;

    let observation = make_observation(
        "44444444-4444-4444-4444-444444444444",
        DEFAULT_ORG_ID,
        "memory list regression probe",
        &["regression"],
        1_700_100_001,
        "ses-list",
    );
    repo.store_observation(&observation).await?;

    let results = repo.search(DEFAULT_ORG_ID, "", 10).await?;
    assert!(
        results
            .iter()
            .any(|item| item.id == "44444444-4444-4444-4444-444444444444")
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn observation_repo_scopes_duplicate_content_hash_by_org() -> TestResult {
    let repo = setup_repo().await?;
    let other_org_id = "org-tenant-b";
    let primary = make_observation(
        "55555555-5555-5555-5555-555555555555",
        DEFAULT_ORG_ID,
        "shared tenant content",
        &["shared"],
        1_700_200_001,
        "ses-tenant",
    );
    let other = make_observation(
        "66666666-6666-6666-6666-666666666666",
        other_org_id,
        "shared tenant content",
        &["shared"],
        1_700_200_002,
        "ses-tenant",
    );

    repo.store_observation(&primary).await?;
    repo.store_observation(&other).await?;

    assert!(
        repo.get_observation(
            DEFAULT_ORG_ID,
            &ObservationId::from_string("66666666-6666-6666-6666-666666666666"),
        )
        .await?
        .is_none()
    );
    assert!(
        repo.get_observation(
            other_org_id,
            &ObservationId::from_string("55555555-5555-5555-5555-555555555555"),
        )
        .await?
        .is_none()
    );

    let primary_search = repo.search(DEFAULT_ORG_ID, "shared", 10).await?;
    assert!(
        primary_search
            .iter()
            .any(|item| item.id == "55555555-5555-5555-5555-555555555555")
    );
    assert!(
        primary_search
            .iter()
            .all(|item| item.id != "66666666-6666-6666-6666-666666666666")
    );

    let other_search = repo.search(other_org_id, "shared", 10).await?;
    assert!(
        other_search
            .iter()
            .any(|item| item.id == "66666666-6666-6666-6666-666666666666")
    );
    assert!(
        other_search
            .iter()
            .all(|item| item.id != "55555555-5555-5555-5555-555555555555")
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn session_summary_lookup_is_scoped_by_org() -> TestResult {
    let repo = setup_repo().await?;
    let other_org_id = "org-tenant-b";
    let session_id = "77777777-7777-7777-7777-777777777777";
    let summary = SessionSummary {
        id: "summary-1".to_owned(),
        project_id: "proj-memory-summary".to_owned(),
        org_id: DEFAULT_ORG_ID.to_owned(),
        session_id: session_id.to_owned(),
        topics: vec!["topic".to_owned()],
        decisions: Vec::new(),
        next_steps: Vec::new(),
        key_files: Vec::new(),
        origin_context: None,
        created_at: 1_700_300_001,
    };

    repo.store_session_summary(&summary).await?;

    assert!(
        repo.get_session_summary(other_org_id, &SessionId::from_string(session_id))
            .await?
            .is_none()
    );
    let fetched = repo
        .get_session_summary(DEFAULT_ORG_ID, &SessionId::from_string(session_id))
        .await?
        .ok_or("summary should exist for owning org")?;
    assert_eq!(fetched.id, "summary-1");
    Ok(())
}
