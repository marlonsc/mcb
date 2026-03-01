//! Dashboard query tests (T9) — `SeaOrmDashboardAdapter` validation.
//!
//! Tests use in-memory `SQLite` with migrations, inserting test data and
//! verifying aggregation queries return correct results.

use mcb_domain::ports::DashboardQueryPort;
use mcb_domain::test_utils::TestResult;
use mcb_providers::database::seaorm::dashboard::SeaOrmDashboardAdapter;
use mcb_providers::migration::Migrator;
use rstest::rstest;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

/// Setup helper — create in-memory `SQLite` with migrations.
async fn setup() -> TestResult<DatabaseConnection> {
    let db = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

/// Insert test observations with specific timestamps.
async fn insert_observation(db: &DatabaseConnection, id: &str, created_at: i64) -> TestResult {
    let sql = format!(
        "INSERT INTO observations (id, project_id, content, content_hash, tags, observation_type, metadata, created_at) \
         VALUES ('{id}', 'proj-1', 'content-{id}', 'hash-{id}', '[]', 'code', '{{}}', {created_at})"
    );
    db.execute_unprepared(&sql).await?;
    Ok(())
}

/// Insert test `tool_calls`.
async fn insert_tool_call(db: &DatabaseConnection, id: &str, tool_name: &str) -> TestResult {
    let sql = format!(
        "INSERT INTO tool_calls (id, org_id, project_id, repo_id, session_id, tool_name, params_summary, success, duration_ms, created_at) \
         VALUES ('{id}', NULL, NULL, NULL, 'sess-1', '{tool_name}', NULL, 1, 100, 1)"
    );
    db.execute_unprepared(&sql).await?;
    Ok(())
}

/// Insert test `agent_sessions`.
async fn insert_agent_session(db: &DatabaseConnection, id: &str, agent_type: &str) -> TestResult {
    let sql = format!(
        "INSERT INTO agent_sessions (id, project_id, worktree_id, session_summary_id, agent_type, model, started_at, status) \
         VALUES ('{id}', 'proj-1', 'wt-1', 'sess-summary-{id}', '{agent_type}', 'gpt-4', 1, 'active')"
    );
    db.execute_unprepared(&sql).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Observations by month/day
// ---------------------------------------------------------------------------

#[rstest]
#[tokio::test]
async fn test_observations_by_month_empty_db() -> TestResult {
    let db = setup().await?;
    let adapter = SeaOrmDashboardAdapter::new(db);

    let result = adapter.get_observations_by_month(10).await?;
    assert!(result.is_empty(), "should return empty vec");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_observations_by_month_with_data() -> TestResult {
    let db = setup().await?;

    // Insert observations across different months
    // 2025-01-15 (unix epoch: 1736910000)
    insert_observation(&db, "obs-1", 1_736_910_000).await?;
    insert_observation(&db, "obs-2", 1_736_996_400).await?; // same month
    insert_observation(&db, "obs-3", 1_734_231_600).await?; // 2024-12

    let adapter = SeaOrmDashboardAdapter::new(db);
    let counts = adapter.get_observations_by_month(10).await?;

    // Should have 2 months aggregated
    assert_eq!(counts.len(), 2, "should have 2 months");

    // Most recent month should have 2 observations (sorted DESC)
    assert_eq!(counts[0].count, 2, "first month should have 2 obs");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_observations_by_day_with_data() -> TestResult {
    let db = setup().await?;

    // Insert observations on different days
    insert_observation(&db, "obs-1", 1_736_910_000).await?; // day 1
    insert_observation(&db, "obs-2", 1_736_910_000).await?; // same day
    insert_observation(&db, "obs-3", 1_736_996_400).await?; // day 2

    let adapter = SeaOrmDashboardAdapter::new(db);
    let counts = adapter.get_observations_by_day(10).await?;

    // Should have 2 days aggregated
    assert_eq!(counts.len(), 2, "should have 2 days");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_observations_by_day_respects_limit() -> TestResult {
    let db = setup().await?;

    // Insert observations across many days
    for i in 0..10 {
        let ts = 1_736_910_000 + (i * 86_400); // +1 day each
        insert_observation(&db, &format!("obs-{i}"), ts).await?;
    }

    let adapter = SeaOrmDashboardAdapter::new(db);
    let counts = adapter.get_observations_by_day(5).await?;

    // Limit of 5 should be respected
    assert_eq!(counts.len(), 5, "should respect limit of 5");
    Ok(())
}

// ---------------------------------------------------------------------------
// Tool call counts
// ---------------------------------------------------------------------------

#[rstest]
#[tokio::test]
async fn test_tool_call_counts_empty_db() -> TestResult {
    let db = setup().await?;
    let adapter = SeaOrmDashboardAdapter::new(db);

    let result = adapter.get_tool_call_counts().await?;
    assert!(result.is_empty(), "should return empty vec");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_tool_call_counts_aggregates_by_name() -> TestResult {
    let db = setup().await?;

    // Insert tool calls for different tools
    insert_tool_call(&db, "tc-1", "search_code").await?;
    insert_tool_call(&db, "tc-2", "search_code").await?;
    insert_tool_call(&db, "tc-3", "index").await?;

    let adapter = SeaOrmDashboardAdapter::new(db);
    let counts = adapter.get_tool_call_counts().await?;

    // Should have 2 tool names
    assert_eq!(counts.len(), 2, "should have 2 tool names");

    // Find search_code count
    let search_count = counts
        .iter()
        .find(|c| c.tool_name == "search_code")
        .map_or(0, |c| c.count);
    assert_eq!(search_count, 2, "search_code should have 2 calls");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_tool_call_counts_ordered_by_count_desc() -> TestResult {
    let db = setup().await?;

    // Insert more calls for "index" than "search"
    for i in 0..5 {
        insert_tool_call(&db, &format!("tc-index-{i}"), "index").await?;
    }
    for i in 0..2 {
        insert_tool_call(&db, &format!("tc-search-{i}"), "search").await?;
    }

    let adapter = SeaOrmDashboardAdapter::new(db);
    let counts = adapter.get_tool_call_counts().await?;

    // First item should be "index" with higher count
    assert_eq!(counts[0].tool_name, "index");
    assert_eq!(counts[0].count, 5);
    Ok(())
}

// ---------------------------------------------------------------------------
// Agent session stats
// ---------------------------------------------------------------------------

#[rstest]
#[tokio::test]
async fn test_agent_session_stats_empty_db() -> TestResult {
    let db = setup().await?;
    let adapter = SeaOrmDashboardAdapter::new(db);

    let stats = adapter.get_agent_session_stats().await?;
    assert_eq!(stats.total_sessions, 0, "total_sessions should be 0");
    assert_eq!(stats.total_agents, 0, "total_agents should be 0");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_agent_session_stats_counts_sessions() -> TestResult {
    let db = setup().await?;

    // Insert 3 sessions
    insert_agent_session(&db, "s1", "claude").await?;
    insert_agent_session(&db, "s2", "gpt-4").await?;
    insert_agent_session(&db, "s3", "claude").await?;

    let adapter = SeaOrmDashboardAdapter::new(db);
    let stats = adapter.get_agent_session_stats().await?;

    assert_eq!(stats.total_sessions, 3, "total_sessions should be 3");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_agent_session_stats_counts_unique_agents() -> TestResult {
    let db = setup().await?;

    // Insert sessions with 2 unique agent types
    insert_agent_session(&db, "s1", "claude").await?;
    insert_agent_session(&db, "s2", "gpt-4").await?;
    insert_agent_session(&db, "s3", "claude").await?; // duplicate agent type

    let adapter = SeaOrmDashboardAdapter::new(db);
    let stats = adapter.get_agent_session_stats().await?;

    assert_eq!(
        stats.total_agents, 2,
        "total_agents should be 2 (unique types)"
    );
    Ok(())
}
