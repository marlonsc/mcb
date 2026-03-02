use std::sync::Arc;

use mcb_domain::ports::AgentEventRepository;
use mcb_domain::utils::tests::utils::TestResult;
use mcb_providers::database::seaorm::entities::{agent_session, delegation, tool_call};
use mcb_providers::database::seaorm::repos::agent::SeaOrmAgentRepository;
use rstest::rstest;
use sea_orm::{ConnectionTrait, Database, EntityTrait};

fn sample_tool_call(session_id: &str) -> mcb_domain::entities::agent::ToolCall {
    mcb_domain::entities::agent::ToolCall {
        id: "tc-test-001".to_owned(),
        session_id: session_id.to_owned(),
        tool_name: "search".to_owned(),
        params_summary: Some("query=agent repo".to_owned()),
        success: true,
        error_message: None,
        duration_ms: Some(12),
        created_at: 1_700_000_000,
    }
}

fn sample_delegation(
    parent_session_id: &str,
    child_session_id: &str,
) -> mcb_domain::entities::agent::Delegation {
    mcb_domain::entities::agent::Delegation {
        id: "del-test-001".to_owned(),
        parent_session_id: parent_session_id.to_owned(),
        child_session_id: child_session_id.to_owned(),
        prompt: "Implement task".to_owned(),
        prompt_embedding_id: Some("emb-1".to_owned()),
        result: Some("Done".to_owned()),
        success: true,
        created_at: 1_700_000_010,
        completed_at: Some(1_700_000_020),
        duration_ms: Some(10),
    }
}

async fn setup_repo() -> TestResult<(SeaOrmAgentRepository, Arc<sea_orm::DatabaseConnection>)> {
    let db = Arc::new(Database::connect("sqlite::memory:").await?);
    db.execute_unprepared("PRAGMA foreign_keys = ON").await?;
    mcb_domain::registry::database::migrate_up(Box::new((*db).clone()), None).await?;

    // Seed required parent data for FK constraints
    db.execute_unprepared(
        "INSERT INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES ('org-1', 'Test Org', 'test-org', '{}', 1, 1)",
    ).await?;
    db.execute_unprepared(
        "INSERT INTO projects (id, org_id, name, path, created_at, updated_at) VALUES ('proj-agent', 'org-1', 'Agent Project', '/tmp/agent', 1, 1)",
    ).await?;

    Ok((SeaOrmAgentRepository::new(Arc::clone(&db)), db))
}

#[ignore = "auto-create session on orphan tool_call not yet implemented"]
#[rstest]
#[tokio::test]
async fn log_tool_persists_when_session_missing() -> TestResult {
    let (repo, db) = setup_repo().await?;
    let tool = sample_tool_call("ses-tool-missing");

    repo.store_tool_call(&tool).await?;

    let stored_tool = tool_call::Entity::find_by_id(tool.id.clone())
        .one(db.as_ref())
        .await?
        .ok_or("tool_call not stored")?;

    assert_eq!(stored_tool.session_id, "ses-tool-missing");
    assert_eq!(stored_tool.tool_name, "search");
    assert_eq!(stored_tool.success, 1);

    let created_session = agent_session::Entity::find_by_id("ses-tool-missing")
        .one(db.as_ref())
        .await?
        .ok_or("session should be auto-created")?;

    assert_eq!(created_session.id, "ses-tool-missing");

    Ok(())
}

#[ignore = "auto-create session on orphan delegation not yet implemented"]
#[rstest]
#[tokio::test]
async fn log_delegation_persists_when_sessions_missing() -> TestResult {
    let (repo, db) = setup_repo().await?;
    let event = sample_delegation("ses-parent-missing", "ses-child-missing");

    repo.store_delegation(&event).await?;

    let stored_delegation = delegation::Entity::find_by_id(event.id.clone())
        .one(db.as_ref())
        .await?
        .ok_or("delegation not stored")?;

    assert_eq!(stored_delegation.parent_session_id, "ses-parent-missing");
    assert_eq!(stored_delegation.child_session_id, "ses-child-missing");
    assert_eq!(stored_delegation.success, 1);

    let parent = agent_session::Entity::find_by_id("ses-parent-missing")
        .one(db.as_ref())
        .await?
        .ok_or("parent session should be auto-created")?;
    let child = agent_session::Entity::find_by_id("ses-child-missing")
        .one(db.as_ref())
        .await?
        .ok_or("child session should be auto-created")?;

    assert_eq!(parent.id, "ses-parent-missing");
    assert_eq!(child.id, "ses-child-missing");

    Ok(())
}
