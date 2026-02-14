use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::ports::infrastructure::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::MemoryRepository;
use mcb_domain::value_objects::{ObservationId, SessionId};
use mcb_providers::database::create_memory_repository_with_executor;
use rstest::rstest;

async fn setup_memory_repo() -> (
    std::sync::Arc<dyn MemoryRepository>,
    std::sync::Arc<dyn DatabaseExecutor>,
    tempfile::TempDir,
) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let (memory_repo, executor) = create_memory_repository_with_executor(db_path)
        .await
        .expect("create memory repository");

    seed_default_org_and_project(executor.as_ref(), "proj-schema-upgrade").await;

    (memory_repo, executor, temp_dir)
}

async fn seed_default_org_and_project(executor: &dyn DatabaseExecutor, project_id: &str) {
    executor
        .execute(
            "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(DEFAULT_ORG_ID.to_string()),
                SqlParam::String("default".to_string()),
                SqlParam::String("default".to_string()),
                SqlParam::String("{}".to_string()),
                SqlParam::I64(0),
                SqlParam::I64(0),
            ],
        )
        .await
        .expect("seed default org");

    executor
        .execute(
            "INSERT OR IGNORE INTO projects (id, org_id, name, path, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(project_id.to_string()),
                SqlParam::String(DEFAULT_ORG_ID.to_string()),
                SqlParam::String("schema-upgrade-project".to_string()),
                SqlParam::String("/tmp/schema-upgrade-project".to_string()),
                SqlParam::I64(0),
                SqlParam::I64(0),
            ],
        )
        .await
        .expect("seed project");
}

#[rstest]
#[case("observation_metadata")]
#[case("session_origin_context")]
#[case("observation_tags")]
#[case("session_topics")]
#[tokio::test]
async fn malformed_json_is_rejected(#[case] scenario: &str) {
    let (memory_repo, executor, _temp_dir) = setup_memory_repo().await;

    match scenario {
        "observation_metadata" => {
            let obs_id = ObservationId::from_name("obs-malformed");
            executor
                .execute(
                    "INSERT INTO observations (id, project_id, content, content_hash, tags, observation_type, metadata, created_at, embedding_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    &[
                        SqlParam::String(obs_id.to_string()),
                        SqlParam::String("proj-schema-upgrade".to_string()),
                        SqlParam::String("content".to_string()),
                        SqlParam::String("hash-obs-malformed".to_string()),
                        SqlParam::String("[]".to_string()),
                        SqlParam::String("context".to_string()),
                        SqlParam::String("{not-valid-json".to_string()),
                        SqlParam::I64(1),
                        SqlParam::Null,
                    ],
                )
                .await
                .expect("insert malformed observation row");

            let err = memory_repo
                .get_observation(&obs_id)
                .await
                .expect_err("malformed metadata must fail");
            assert!(
                err.to_string()
                    .contains("invalid observation metadata JSON"),
                "unexpected error: {err}"
            );
        }
        "session_origin_context" => {
            let session_id = SessionId::from_name("session-malformed");
            let summary_id = ObservationId::from_name("summary-malformed");
            executor
                .execute(
                    "INSERT INTO session_summaries (id, project_id, session_id, topics, decisions, next_steps, key_files, origin_context, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    &[
                        SqlParam::String(summary_id.to_string()),
                        SqlParam::String("proj-schema-upgrade".to_string()),
                        SqlParam::String(session_id.to_string()),
                        SqlParam::String("[]".to_string()),
                        SqlParam::String("[]".to_string()),
                        SqlParam::String("[]".to_string()),
                        SqlParam::String("[]".to_string()),
                        SqlParam::String("{not-valid-json".to_string()),
                        SqlParam::I64(1),
                    ],
                )
                .await
                .expect("insert malformed session summary row");

            let err = memory_repo
                .get_session_summary(&session_id)
                .await
                .expect_err("malformed origin_context must fail");
            assert!(
                err.to_string()
                    .contains("invalid session summary origin_context JSON"),
                "unexpected error: {err}"
            );
        }
        "observation_tags" => {
            let obs_id = ObservationId::from_name("obs-bad-tags");
            executor
                .execute(
                    "INSERT INTO observations (id, project_id, content, content_hash, tags, observation_type, metadata, created_at, embedding_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    &[
                        SqlParam::String(obs_id.to_string()),
                        SqlParam::String("proj-schema-upgrade".to_string()),
                        SqlParam::String("content".to_string()),
                        SqlParam::String("hash-obs-bad-tags".to_string()),
                        SqlParam::String("{bad-tags-json".to_string()),
                        SqlParam::String("context".to_string()),
                        SqlParam::String("{}".to_string()),
                        SqlParam::I64(1),
                        SqlParam::Null,
                    ],
                )
                .await
                .expect("insert malformed tags row");

            let err = memory_repo
                .get_observation(&obs_id)
                .await
                .expect_err("malformed tags must fail");
            assert!(
                err.to_string().contains("invalid observation tags JSON"),
                "unexpected error: {err}"
            );
        }
        "session_topics" => {
            let session_id = SessionId::from_name("session-bad-topics");
            let summary_id = ObservationId::from_name("summary-bad-topics");
            executor
                .execute(
                    "INSERT INTO session_summaries (id, project_id, session_id, topics, decisions, next_steps, key_files, origin_context, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    &[
                        SqlParam::String(summary_id.to_string()),
                        SqlParam::String("proj-schema-upgrade".to_string()),
                        SqlParam::String(session_id.to_string()),
                        SqlParam::String("{bad-topics-json".to_string()),
                        SqlParam::String("[]".to_string()),
                        SqlParam::String("[]".to_string()),
                        SqlParam::String("[]".to_string()),
                        SqlParam::String("null".to_string()),
                        SqlParam::I64(1),
                    ],
                )
                .await
                .expect("insert malformed topics row");

            let err = memory_repo
                .get_session_summary(&session_id)
                .await
                .expect_err("malformed topics must fail");
            assert!(
                err.to_string()
                    .contains("invalid session summary topics JSON"),
                "unexpected error: {err}"
            );
        }
        _ => panic!("unknown scenario: {scenario}"),
    }
}
