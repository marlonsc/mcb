use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use sea_orm_migration::MigratorTrait;

use mcb_domain::utils::tests::utils::TestResult;
use mcb_providers::database::seaorm::migration::Migrator;
use rstest::rstest;

async fn query_names(db: &DatabaseConnection, sql: &str) -> TestResult<Vec<String>> {
    let stmt = Statement::from_string(DatabaseBackend::Sqlite, sql);
    let rows = db.query_all_raw(stmt).await?;
    let mut names = Vec::with_capacity(rows.len());
    for r in &rows {
        names.push(r.try_get_by_index::<String>(0)?);
    }
    Ok(names)
}

#[rstest]
#[tokio::test]
async fn migration_creates_all_tables() -> TestResult {
    let db = sea_orm::Database::connect("sqlite::memory:").await?;

    Migrator::up(&db, None).await?;

    let table_names = query_names(
        &db,
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != 'seaql_migrations' ORDER BY name",
    ).await?;

    let expected = [
        "agent_sessions",
        "agent_worktree_assignments",
        "api_keys",
        "branches",
        "checkpoints",
        "collections",
        "delegations",
        "error_pattern_matches",
        "error_patterns",
        "file_hashes",
        "index_operations",
        "issue_comments",
        "issue_label_assignments",
        "issue_labels",
        "observations",
        "observations_fts",
        "organizations",
        "plan_reviews",
        "plan_versions",
        "plans",
        "project_issues",
        "projects",
        "repositories",
        "session_summaries",
        "team_members",
        "teams",
        "tool_calls",
        "users",
        "worktrees",
    ];

    for name in &expected {
        assert!(
            table_names.iter().any(|t| t == name),
            "missing table: {name} (found: {table_names:?})"
        );
    }
    Ok(())
}

#[rstest]
#[tokio::test]
async fn migration_creates_fts5_triggers() -> TestResult {
    let db = sea_orm::Database::connect("sqlite::memory:").await?;

    Migrator::up(&db, None).await?;

    let trigger_names = query_names(
        &db,
        "SELECT name FROM sqlite_master WHERE type='trigger' ORDER BY name",
    )
    .await?;

    assert!(
        trigger_names.iter().any(|t| t == "obs_ai"),
        "missing trigger obs_ai"
    );
    assert!(
        trigger_names.iter().any(|t| t == "obs_ad"),
        "missing trigger obs_ad"
    );
    assert!(
        trigger_names.iter().any(|t| t == "obs_au"),
        "missing trigger obs_au"
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn migration_creates_indexes() -> TestResult {
    let db = sea_orm::Database::connect("sqlite::memory:").await?;

    Migrator::up(&db, None).await?;

    let index_names = query_names(
        &db,
        "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%' ORDER BY name",
    )
    .await?;

    assert!(
        index_names.iter().any(|i| i == "idx_obs_project"),
        "missing index idx_obs_project"
    );
    assert!(
        index_names.iter().any(|i| i == "idx_organizations_name"),
        "missing index idx_organizations_name"
    );
    assert!(
        index_names.iter().any(|i| i == "idx_branches_repo"),
        "missing index idx_branches_repo"
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn migration_down_drops_all_tables() -> TestResult {
    let db = sea_orm::Database::connect("sqlite::memory:").await?;

    Migrator::up(&db, None).await?;
    Migrator::down(&db, None).await?;

    let table_names = query_names(
        &db,
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != 'seaql_migrations'",
    ).await?;

    assert!(
        table_names.is_empty(),
        "tables should be empty after down migration, found: {table_names:?}"
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn migration_is_idempotent() -> TestResult {
    let db = sea_orm::Database::connect("sqlite::memory:").await?;

    Migrator::up(&db, None).await?;
    Migrator::up(&db, None).await?;
    Ok(())
}
