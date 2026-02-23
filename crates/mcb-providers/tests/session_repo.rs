#![allow(clippy::expect_used, missing_docs)]

use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::agent::{AgentSession, AgentSessionStatus, AgentType};
use mcb_domain::ports::{AgentSessionQuery, AgentSessionRepository};
use mcb_providers::database::seaorm::migration::Migrator;
use mcb_providers::database::seaorm::repos::SeaOrmAgentSessionRepository;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use mcb_providers::database::seaorm::entities::{organization, project};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

async fn setup_db() -> TestResult<Arc<DatabaseConnection>> {
    let db = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;
    Ok(Arc::new(db))
}

async fn seed_org_and_project(db: &DatabaseConnection, project_id: &str) -> TestResult {
    let org = organization::ActiveModel {
        id: Set(DEFAULT_ORG_ID.to_owned()),
        name: Set("Default Org".to_owned()),
        slug: Set("default-org".to_owned()),
        settings_json: Set("{}".to_owned()),
        created_at: Set(1_700_000_000),
        updated_at: Set(1_700_000_000),
    };
    org.insert(db).await?;

    let proj = project::ActiveModel {
        id: Set(project_id.to_owned()),
        org_id: Set(DEFAULT_ORG_ID.to_owned()),
        name: Set("Session Repo Project".to_owned()),
        path: Set("/tmp/session-repo".to_owned()),
        created_at: Set(1_700_000_000),
        updated_at: Set(1_700_000_000),
    };
    proj.insert(db).await?;

    Ok(())
}

fn sample_session() -> AgentSession {
    AgentSession {
        id: "ses-001".to_owned(),
        session_summary_id: "sum-001".to_owned(),
        agent_type: AgentType::Sisyphus,
        model: "gpt-5.3-codex".to_owned(),
        parent_session_id: None,
        started_at: 1_700_000_001,
        ended_at: None,
        duration_ms: None,
        status: AgentSessionStatus::Active,
        prompt_summary: Some("Build SeaORM repository".to_owned()),
        result_summary: None,
        token_count: Some(42),
        tool_calls_count: Some(1),
        delegations_count: Some(0),
        project_id: Some("proj-001".to_owned()),
        worktree_id: None,
    }
}

#[tokio::test]
async fn create_get_update_list_lifecycle() -> TestResult {
    let db = setup_db().await?;
    seed_org_and_project(db.as_ref(), "proj-001").await?;
    let repo = SeaOrmAgentSessionRepository::new(Arc::clone(&db));

    let mut session = sample_session();
    repo.create_session(&session).await?;

    let created = repo.get_session("ses-001").await?;
    let created = match created {
        Some(value) => value,
        None => return Err("expected created session".into()),
    };
    assert_eq!(created.agent_type, AgentType::Sisyphus);
    assert_eq!(created.status, AgentSessionStatus::Active);

    session.status = AgentSessionStatus::Completed;
    session.ended_at = Some(1_700_000_100);
    session.duration_ms = Some(99_000);
    session.result_summary = Some("Repository implemented".to_owned());
    repo.update_session(&session).await?;

    let updated = repo.get_session("ses-001").await?;
    let updated = match updated {
        Some(value) => value,
        None => return Err("expected updated session".into()),
    };
    assert_eq!(updated.status, AgentSessionStatus::Completed);
    assert_eq!(
        updated.result_summary.as_deref(),
        Some("Repository implemented")
    );

    let listed = repo
        .list_sessions(AgentSessionQuery {
            status: Some(AgentSessionStatus::Completed),
            agent_type: Some(AgentType::Sisyphus),
            project_id: Some("proj-001".to_owned()),
            ..AgentSessionQuery::default()
        })
        .await?;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "ses-001");

    let summary = repo.summarize_session("ses-001").await?;
    let summary = match summary {
        Some(value) => value,
        None => return Err("expected summary".into()),
    };
    assert_eq!(summary.session_id, "ses-001");
    assert_eq!(summary.status, AgentSessionStatus::Completed);

    Ok(())
}

#[tokio::test]
async fn create_rejects_blank_model_schema() -> TestResult {
    let db = setup_db().await?;
    seed_org_and_project(db.as_ref(), "proj-001").await?;
    let repo = SeaOrmAgentSessionRepository::new(Arc::clone(&db));

    let mut session = sample_session();
    session.id = "ses-blank-model".to_owned();
    session.model = "   ".to_owned();

    let error = repo
        .create_session(&session)
        .await
        .expect_err("blank model must fail schema validation");
    assert!(error.to_string().contains("model"));

    Ok(())
}

#[tokio::test]
async fn get_rejects_invalid_agent_type_schema() -> TestResult {
    let db = setup_db().await?;
    seed_org_and_project(db.as_ref(), "proj-001").await?;
    let repo = SeaOrmAgentSessionRepository::new(Arc::clone(&db));

    let mut session = sample_session();
    session.id = "ses-invalid-type".to_owned();
    repo.create_session(&session).await?;

    db.execute_unprepared(
        "UPDATE agent_sessions SET agent_type = 'bad-agent-type' WHERE id = 'ses-invalid-type'",
    )
    .await?;

    let error = repo
        .get_session("ses-invalid-type")
        .await
        .expect_err("invalid persisted agent_type must fail schema validation");
    assert!(error.to_string().contains("agent_type"));

    Ok(())
}
