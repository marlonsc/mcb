use mcb_domain::schema::{
    MemorySchema, MemorySchemaDdlGenerator, ProjectSchema, SchemaDdlGenerator,
};
use mcb_providers::database::{SqliteMemoryDdlGenerator, SqliteSchemaDdlGenerator};

#[test]
fn test_sqlite_memory_ddl_generator_produces_statements() {
    let generator = SqliteMemoryDdlGenerator;
    let schema = MemorySchema::definition();
    let ddl: Vec<String> = generator.generate_ddl(&schema);
    assert!(!ddl.is_empty());
    assert!(ddl.iter().any(|s| s.contains("CREATE TABLE")));
    assert!(ddl.iter().any(|s| s.contains("fts5")));
}

#[test]
fn test_sqlite_project_schema_ddl_includes_all_entities() {
    let generator = SqliteSchemaDdlGenerator;
    let schema = ProjectSchema::definition();
    let ddl: Vec<String> = generator.generate_ddl(&schema);
    assert!(!ddl.is_empty());
    assert!(
        ddl.iter()
            .any(|s| s.contains("CREATE TABLE") && s.contains("collections"))
    );
    assert!(ddl.iter().any(|s| s.contains("observations")));
    assert!(ddl.iter().any(|s| s.contains("session_summaries")));
    assert!(ddl.iter().any(|s| s.contains("file_hashes")));
    assert!(
        ddl.iter()
            .any(|s| s.contains("UNIQUE(project_id, collection, file_path)"))
    );
    assert!(ddl.iter().any(|s| s.contains("AUTOINCREMENT")));
}

#[test]
fn test_project_schema_has_27_create_table_statements() {
    let generator = SqliteSchemaDdlGenerator;
    let schema = ProjectSchema::definition();
    let ddl: Vec<String> = generator.generate_ddl(&schema);
    let create_count = ddl.iter().filter(|s| s.starts_with("CREATE TABLE")).count();
    assert_eq!(
        create_count, 27,
        "Expected 27 CREATE TABLE statements, got {create_count}"
    );
}

#[test]
fn test_project_schema_contains_all_multi_tenant_tables() {
    let generator = SqliteSchemaDdlGenerator;
    let schema = ProjectSchema::definition();
    let ddl: Vec<String> = generator.generate_ddl(&schema);
    let joined = ddl.join("\n");

    let expected_tables = [
        "organizations",
        "users",
        "teams",
        "team_members",
        "api_keys",
        "projects",
        "collections",
        "observations",
        "session_summaries",
        "file_hashes",
        "agent_sessions",
        "delegations",
        "tool_calls",
        "checkpoints",
        "error_patterns",
        "error_pattern_matches",
        "project_issues",
        "issue_comments",
        "issue_labels",
        "issue_label_assignments",
        "plans",
        "plan_versions",
        "plan_reviews",
        "repositories",
        "branches",
        "worktrees",
        "agent_worktree_assignments",
    ];

    for table in &expected_tables {
        let pattern = format!("CREATE TABLE IF NOT EXISTS {table}");
        assert!(joined.contains(&pattern), "Missing table in DDL: {table}");
    }
}

#[test]
fn test_project_schema_contains_all_fk_references() {
    let generator = SqliteSchemaDdlGenerator;
    let schema = ProjectSchema::definition();
    let ddl: Vec<String> = generator.generate_ddl(&schema);

    let expected_fks = [
        // Core project FKs
        ("collections", "REFERENCES projects(id)"),
        ("observations", "REFERENCES projects(id)"),
        ("session_summaries", "REFERENCES projects(id)"),
        ("file_hashes", "REFERENCES projects(id)"),
        // Multi-tenant FKs
        ("users", "REFERENCES organizations(id)"),
        ("teams", "REFERENCES organizations(id)"),
        ("team_members", "REFERENCES teams(id)"),
        ("team_members", "REFERENCES users(id)"),
        ("api_keys", "REFERENCES users(id)"),
        ("api_keys", "REFERENCES organizations(id)"),
        ("projects", "REFERENCES organizations(id)"),
        // Agent FKs
        ("agent_sessions", "REFERENCES agent_sessions(id)"),
        ("agent_sessions", "REFERENCES projects(id)"),
        ("agent_sessions", "REFERENCES worktrees(id)"),
        ("delegations", "REFERENCES agent_sessions(id)"),
        ("tool_calls", "REFERENCES agent_sessions(id)"),
        ("checkpoints", "REFERENCES agent_sessions(id)"),
        // Error pattern FKs
        ("error_patterns", "REFERENCES projects(id)"),
        ("error_pattern_matches", "REFERENCES error_patterns(id)"),
        ("error_pattern_matches", "REFERENCES observations(id)"),
        // Plan entity FKs
        ("plans", "REFERENCES organizations(id)"),
        ("plans", "REFERENCES projects(id)"),
        ("plans", "REFERENCES users(id)"),
        ("plan_versions", "REFERENCES organizations(id)"),
        ("plan_versions", "REFERENCES plans(id)"),
        ("plan_versions", "REFERENCES users(id)"),
        ("plan_reviews", "REFERENCES organizations(id)"),
        ("plan_reviews", "REFERENCES plan_versions(id)"),
        ("plan_reviews", "REFERENCES users(id)"),
        // Issue entity FKs
        ("project_issues", "REFERENCES organizations(id)"),
        ("project_issues", "REFERENCES projects(id)"),
        ("project_issues", "REFERENCES users(id)"),
        ("project_issues", "REFERENCES project_issues(id)"),
        ("issue_comments", "REFERENCES project_issues(id)"),
        ("issue_comments", "REFERENCES users(id)"),
        ("issue_labels", "REFERENCES organizations(id)"),
        ("issue_labels", "REFERENCES projects(id)"),
        ("issue_label_assignments", "REFERENCES project_issues(id)"),
        ("issue_label_assignments", "REFERENCES issue_labels(id)"),
        // VCS entity FKs
        ("repositories", "REFERENCES organizations(id)"),
        ("repositories", "REFERENCES projects(id)"),
        ("branches", "REFERENCES repositories(id)"),
        ("worktrees", "REFERENCES repositories(id)"),
        ("worktrees", "REFERENCES branches(id)"),
        (
            "agent_worktree_assignments",
            "REFERENCES agent_sessions(id)",
        ),
        ("agent_worktree_assignments", "REFERENCES worktrees(id)"),
    ];

    for (table, fk_ref) in &expected_fks {
        let table_ddl = ddl.iter().find(|s| {
            s.starts_with("CREATE TABLE") && s.contains(&format!("IF NOT EXISTS {table}"))
        });
        assert!(table_ddl.is_some(), "Table {table} not found in DDL");
        let table_ddl = table_ddl.unwrap();
        assert!(
            table_ddl.contains(fk_ref),
            "Missing FK in {table}: expected {fk_ref}\nActual DDL: {table_ddl}"
        );
    }

    let schema_def = ProjectSchema::definition();
    assert_eq!(
        schema_def.foreign_keys.len(),
        47,
        "Expected 47 total FK definitions, got {}",
        schema_def.foreign_keys.len()
    );
}

#[test]
fn test_project_schema_contains_all_indexes() {
    let generator = SqliteSchemaDdlGenerator;
    let schema = ProjectSchema::definition();
    let ddl: Vec<String> = generator.generate_ddl(&schema);
    let joined = ddl.join("\n");

    let expected_indexes = [
        // Core indexes
        "idx_projects_org",
        "idx_collections_project",
        "idx_obs_project",
        "idx_summary_project",
        "idx_file_hashes_project",
        "idx_file_hashes_collection",
        "idx_file_hashes_deleted",
        // Memory indexes
        "idx_obs_hash",
        "idx_obs_created",
        "idx_obs_type",
        "idx_obs_embedding",
        "idx_summary_session",
        // Agent indexes
        "idx_agent_sessions_summary",
        "idx_agent_sessions_parent",
        "idx_agent_sessions_type",
        "idx_agent_sessions_project",
        "idx_agent_sessions_worktree",
        "idx_agent_sessions_started",
        "idx_delegations_parent",
        "idx_delegations_child",
        "idx_tool_calls_session",
        "idx_tool_calls_tool",
        "idx_checkpoints_session",
        // Error pattern indexes
        "idx_error_patterns_project",
        "idx_error_patterns_category",
        "idx_error_patterns_signature",
        "idx_error_patterns_embedding",
        "idx_error_patterns_last_seen",
        "idx_error_pattern_matches_pattern",
        "idx_error_pattern_matches_observation",
        // Issue entity indexes
        "idx_issues_org",
        "idx_issues_project",
        "idx_issues_phase",
        "idx_issues_status",
        "idx_issues_assignee",
        "idx_issues_parent",
        "idx_issue_comments_issue",
        "idx_issue_comments_author",
        "idx_issue_labels_org",
        "idx_issue_labels_project",
        "idx_issue_label_assignments_issue",
        "idx_issue_label_assignments_label",
        // Multi-tenant indexes
        "idx_users_org",
        "idx_users_email",
        "idx_users_api_key_hash",
        "idx_teams_org",
        "idx_team_members_team",
        "idx_team_members_user",
        "idx_api_keys_user",
        "idx_api_keys_org",
        "idx_api_keys_key_hash",
        "idx_organizations_name",
        // Plan entity indexes
        "idx_plans_org",
        "idx_plans_project",
        "idx_plans_status",
        "idx_plan_versions_org",
        "idx_plan_versions_plan",
        "idx_plan_versions_created_by",
        "idx_plan_reviews_org",
        "idx_plan_reviews_version",
        "idx_plan_reviews_reviewer",
        // VCS entity indexes
        "idx_repositories_org",
        "idx_repositories_project",
        "idx_branches_repo",
        "idx_worktrees_repo",
        "idx_worktrees_branch",
        "idx_worktrees_agent",
        "idx_agent_worktree_assignments_session",
        "idx_agent_worktree_assignments_worktree",
    ];

    let index_count = ddl.iter().filter(|s| s.starts_with("CREATE INDEX")).count();
    assert_eq!(
        index_count,
        expected_indexes.len(),
        "Expected {} indexes, got {index_count}",
        expected_indexes.len()
    );

    for idx_name in &expected_indexes {
        assert!(
            joined.contains(idx_name),
            "Missing index in DDL: {idx_name}"
        );
    }
}

#[test]
fn test_project_schema_contains_unique_constraints() {
    let generator = SqliteSchemaDdlGenerator;
    let schema = ProjectSchema::definition();
    let ddl: Vec<String> = generator.generate_ddl(&schema);
    let joined = ddl.join("\n");

    let expected_uniques = [
        "UNIQUE(org_id, name)",                      // projects
        "UNIQUE(project_id, name)",                  // collections
        "UNIQUE(project_id, collection, file_path)", // file_hashes
        "UNIQUE(org_id, email)",                     // users
        "UNIQUE(team_id, user_id)",                  // team_members
        "UNIQUE(issue_id, label_id)",                // issue_label_assignments
        "UNIQUE(plan_id, version_number)",           // plan_versions
        "UNIQUE(org_id, project_id, name)",          // repositories
        "UNIQUE(repository_id, name)",               // branches
    ];

    for uc in &expected_uniques {
        assert!(
            joined.contains(uc),
            "Missing unique constraint in DDL: {uc}"
        );
    }

    // teams has UNIQUE(org_id, name) which duplicates the projects one, so check table-level
    let teams_ddl = ddl
        .iter()
        .find(|s| s.contains("IF NOT EXISTS teams ("))
        .expect("teams table not found");
    assert!(
        teams_ddl.contains("UNIQUE(org_id, name)"),
        "teams table missing UNIQUE(org_id, name)"
    );
}

#[tokio::test]
async fn test_ddl_executes_against_fresh_sqlite_db() {
    use mcb_providers::database::create_memory_repository_with_executor;

    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("ddl_test.db");

    let (_repo, executor) = create_memory_repository_with_executor(db_path)
        .await
        .expect("DDL execution failed — schema is not valid SQLite");

    let rows = executor
        .query_all(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
            &[],
        )
        .await
        .expect("Failed to query sqlite_master");

    let table_names: Vec<String> = rows
        .iter()
        .filter_map(|r| r.try_get_string("name").ok().flatten())
        .collect();

    let expected_tables = [
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
        "teams",
        "team_members",
        "tool_calls",
        "worktrees",
    ];

    assert!(
        table_names.len() >= expected_tables.len(),
        "Expected at least {} tables, found {}: {:?}",
        expected_tables.len(),
        table_names.len(),
        table_names
    );

    for expected in &expected_tables {
        assert!(
            table_names.contains(&expected.to_string()),
            "Table {expected} not created in SQLite DB. Found: {table_names:?}"
        );
    }

    let index_rows = executor
        .query_all(
            "SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'",
            &[],
        )
        .await
        .expect("Failed to query indexes from sqlite_master");

    assert!(
        index_rows.len() >= 50,
        "Expected at least 50 indexes, found {}",
        index_rows.len()
    );
}
