use mcb_domain::entities::memory::{
    MemoryFilter, Observation, ObservationMetadata, ObservationType,
};
use mcb_domain::ports::MemoryRepository;
use mcb_domain::utils::id::compute_content_hash;
use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::value_objects::ObservationId;
use mcb_providers::database::seaorm::repos::SeaOrmObservationRepository;
use rstest::rstest;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};

fn make_observation(
    id: &str,
    content: &str,
    tags: &[&str],
    created_at: i64,
    session_id: &str,
) -> Observation {
    Observation {
        id: id.to_owned(),
        project_id: "proj-memory".to_owned(),
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
    let db: DatabaseConnection = Database::connect("sqlite::memory:").await?;

    mcb_domain::registry::database::migrate_up(Box::new(db.clone()), None).await?;

    // Seed required parent data
    db.execute_unprepared(
        "INSERT INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES ('org-1', 'Test Org', 'test-org', '{}', 1, 1)",
    ).await?;
    db.execute_unprepared(
        "INSERT INTO projects (id, org_id, name, path, created_at, updated_at) VALUES ('proj-memory', 'org-1', 'Memory Project', '/tmp/proj', 1, 1)",
    ).await?;

    Ok(SeaOrmObservationRepository::new(db))
}

#[rstest]
#[tokio::test]
async fn observation_repo_round_trip_store_get_list_timeline_and_inject() -> TestResult {
    let repo = setup_repo().await?;

    let obs1 = make_observation(
        "11111111-1111-1111-1111-111111111111",
        "first observation content",
        &["important", "alpha"],
        1_700_000_001,
        "ses-1",
    );
    let obs2 = make_observation(
        "22222222-2222-2222-2222-222222222222",
        "second observation content",
        &["important", "beta"],
        1_700_000_002,
        "ses-1",
    );
    let obs3 = make_observation(
        "33333333-3333-3333-3333-333333333333",
        "third observation content",
        &["gamma"],
        1_700_000_003,
        "ses-2",
    );

    repo.store_observation(&obs1).await?;
    repo.store_observation(&obs2).await?;
    repo.store_observation(&obs3).await?;

    let fetched = repo
        .get_observation(&ObservationId::from_string(
            "22222222-2222-2222-2222-222222222222",
        ))
        .await?
        .ok_or("obs2 should exist")?;
    assert_eq!(fetched.id, "22222222-2222-2222-2222-222222222222");
    assert_eq!(fetched.content, "second observation content");

    let filtered = repo
        .list_observations(
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

    let timeline = repo
        .get_timeline(
            &ObservationId::from_string("22222222-2222-2222-2222-222222222222"),
            1,
            1,
            None,
        )
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
        "memory list regression probe",
        &["regression"],
        1_700_100_001,
        "ses-list",
    );
    repo.store_observation(&observation).await?;

    let results = repo.search("", 10).await?;
    assert!(
        results
            .iter()
            .any(|item| item.id == "44444444-4444-4444-4444-444444444444")
    );
    Ok(())
}
