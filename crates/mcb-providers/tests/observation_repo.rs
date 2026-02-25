#![allow(clippy::expect_used, missing_docs)]

use mcb_domain::entities::memory::{
    MemoryFilter, Observation, ObservationMetadata, ObservationType,
};
use mcb_domain::ports::MemoryRepository;
use mcb_domain::utils::compute_content_hash;
use mcb_domain::value_objects::ObservationId;
use mcb_providers::database::seaorm::repos::SeaOrmObservationRepository;
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

async fn setup_repo() -> SeaOrmObservationRepository {
    let db: DatabaseConnection = Database::connect("sqlite::memory:")
        .await
        .expect("connect sqlite memory");

    let schema = [
        "CREATE TABLE organizations (id TEXT PRIMARY KEY, name TEXT NOT NULL, slug TEXT NOT NULL UNIQUE, settings_json TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL)",
        "CREATE TABLE projects (id TEXT PRIMARY KEY, org_id TEXT NOT NULL, name TEXT NOT NULL, path TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL)",
        "CREATE TABLE observations (id TEXT PRIMARY KEY, project_id TEXT NOT NULL, content TEXT NOT NULL, content_hash TEXT NOT NULL UNIQUE, tags TEXT, observation_type TEXT, metadata TEXT, created_at INTEGER NOT NULL, embedding_id TEXT)",
        "CREATE TABLE session_summaries (id TEXT PRIMARY KEY, org_id TEXT, project_id TEXT NOT NULL, repo_id TEXT, session_id TEXT NOT NULL, topics TEXT, decisions TEXT, next_steps TEXT, key_files TEXT, origin_context TEXT, created_at INTEGER NOT NULL)",
        "CREATE VIRTUAL TABLE observations_fts USING fts5(id UNINDEXED, content)",
        "CREATE TRIGGER obs_ai AFTER INSERT ON observations BEGIN INSERT INTO observations_fts(rowid, id, content) VALUES (new.rowid, new.id, new.content); END;",
        "CREATE TRIGGER obs_ad AFTER DELETE ON observations BEGIN DELETE FROM observations_fts WHERE rowid = old.rowid; END;",
        "CREATE TRIGGER obs_au AFTER UPDATE ON observations BEGIN DELETE FROM observations_fts WHERE rowid = old.rowid; INSERT INTO observations_fts(rowid, id, content) VALUES (new.rowid, new.id, new.content); END;",
    ];

    for stmt in schema {
        db.execute_unprepared(stmt)
            .await
            .expect("create observation schema");
    }

    SeaOrmObservationRepository::new(db)
}

#[tokio::test]
async fn observation_repo_round_trip_store_get_list_timeline_and_inject() {
    let repo = setup_repo().await;

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

    repo.store_observation(&obs1).await.expect("store obs1");
    repo.store_observation(&obs2).await.expect("store obs2");
    repo.store_observation(&obs3).await.expect("store obs3");

    let fetched = repo
        .get_observation(&ObservationId::from_string(
            "22222222-2222-2222-2222-222222222222",
        ))
        .await
        .expect("get obs2")
        .expect("obs2 exists");
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
        .await
        .expect("list with tags and session");

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
        .await
        .expect("timeline");
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
        .await
        .expect("inject observations");
    assert!(!injected.is_empty());
    assert!(
        injected
            .iter()
            .all(|obs| obs.tags.iter().any(|tag| tag == "important"))
    );
}

#[tokio::test]
async fn search_handles_empty_query_for_memory_list_bug_regression() {
    let repo = setup_repo().await;

    let observation = make_observation(
        "44444444-4444-4444-4444-444444444444",
        "memory list regression probe",
        &["regression"],
        1_700_100_001,
        "ses-list",
    );
    repo.store_observation(&observation)
        .await
        .expect("store regression observation");

    let results = repo.search("", 10).await.expect("search empty query");
    assert!(
        results
            .iter()
            .any(|item| item.id == "44444444-4444-4444-4444-444444444444")
    );
}
