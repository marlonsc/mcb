use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS organizations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                slug TEXT NOT NULL UNIQUE,
                settings_json TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS repositories (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                local_path TEXT NOT NULL,
                vcs_type TEXT NOT NULL,
                origin_context TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS branches (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                project_id TEXT,
                repository_id TEXT NOT NULL,
                name TEXT NOT NULL,
                is_default INTEGER NOT NULL,
                head_commit TEXT NOT NULL,
                upstream TEXT,
                origin_context TEXT,
                created_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS worktrees (
                id TEXT PRIMARY KEY,
                org_id TEXT,
                project_id TEXT,
                repository_id TEXT NOT NULL,
                branch_id TEXT NOT NULL,
                path TEXT NOT NULL,
                status TEXT NOT NULL,
                assigned_agent_id TEXT,
                origin_context TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS agent_worktree_assignments (
                id TEXT PRIMARY KEY,
                agent_session_id TEXT NOT NULL,
                worktree_id TEXT NOT NULL,
                assigned_at INTEGER NOT NULL,
                released_at INTEGER,
                origin_context TEXT
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                email TEXT NOT NULL,
                display_name TEXT NOT NULL,
                role TEXT NOT NULL,
                api_key_hash TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS teams (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS team_members (
                team_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                role TEXT NOT NULL,
                joined_at INTEGER NOT NULL,
                PRIMARY KEY (team_id, user_id)
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                org_id TEXT NOT NULL,
                key_hash TEXT NOT NULL,
                name TEXT NOT NULL,
                scopes_json TEXT NOT NULL,
                expires_at INTEGER,
                created_at INTEGER NOT NULL,
                revoked_at INTEGER
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS plans (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                created_by TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS plan_versions (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                plan_id TEXT NOT NULL,
                version_number INTEGER NOT NULL,
                content_json TEXT NOT NULL,
                change_summary TEXT NOT NULL,
                created_by TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS plan_reviews (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                plan_version_id TEXT NOT NULL,
                reviewer_id TEXT NOT NULL,
                verdict TEXT NOT NULL,
                feedback TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS project_issues (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                phase_id TEXT,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                issue_type TEXT NOT NULL,
                status TEXT NOT NULL,
                priority INTEGER NOT NULL,
                assignee TEXT,
                labels TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                closed_at INTEGER,
                created_by TEXT NOT NULL,
                estimated_minutes INTEGER,
                actual_minutes INTEGER,
                notes TEXT NOT NULL,
                design TEXT NOT NULL,
                parent_issue_id TEXT,
                closed_reason TEXT NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS project_phases (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                status TEXT NOT NULL,
                started_at INTEGER,
                completed_at INTEGER,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS project_dependencies (
                id TEXT PRIMARY KEY,
                from_issue_id TEXT NOT NULL,
                to_issue_id TEXT NOT NULL,
                dependency_type TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS project_decisions (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                issue_id TEXT,
                title TEXT NOT NULL,
                context TEXT NOT NULL,
                decision TEXT NOT NULL,
                consequences TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS issue_comments (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                author_id TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS issue_labels (
                id TEXT PRIMARY KEY,
                org_id TEXT NOT NULL,
                project_id TEXT NOT NULL,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS issue_label_assignments (
                issue_id TEXT NOT NULL,
                label_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (issue_id, label_id)
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS agent_sessions (
                id TEXT PRIMARY KEY,
                project_id TEXT,
                worktree_id TEXT,
                session_summary_id TEXT NOT NULL,
                agent_type TEXT NOT NULL,
                model TEXT NOT NULL,
                parent_session_id TEXT,
                started_at INTEGER NOT NULL,
                ended_at INTEGER,
                duration_ms INTEGER,
                status TEXT NOT NULL,
                prompt_summary TEXT,
                result_summary TEXT,
                token_count INTEGER,
                tool_calls_count INTEGER,
                delegations_count INTEGER
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS observations (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                content TEXT NOT NULL,
                content_hash TEXT NOT NULL UNIQUE,
                tags TEXT,
                observation_type TEXT,
                metadata TEXT,
                created_at INTEGER NOT NULL,
                embedding_id TEXT
            )",
        )
        .await?;

        // FTS5 virtual table for full-text search on observations
        db.execute_unprepared(
            "CREATE VIRTUAL TABLE IF NOT EXISTS observations_fts USING fts5(id UNINDEXED, content)",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TRIGGER IF NOT EXISTS obs_ai AFTER INSERT ON observations BEGIN INSERT INTO observations_fts(rowid, id, content) VALUES (new.rowid, new.id, new.content); END;",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TRIGGER IF NOT EXISTS obs_ad AFTER DELETE ON observations BEGIN DELETE FROM observations_fts WHERE rowid = old.rowid; END;",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TRIGGER IF NOT EXISTS obs_au AFTER UPDATE ON observations BEGIN DELETE FROM observations_fts WHERE rowid = old.rowid; INSERT INTO observations_fts(rowid, id, content) VALUES (new.rowid, new.id, new.content); END;",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                name TEXT NOT NULL,
                vector_name TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS file_hashes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL,
                collection TEXT NOT NULL,
                file_path TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                indexed_at INTEGER NOT NULL,
                deleted_at INTEGER,
                origin_context TEXT
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS index_operations (
                id TEXT PRIMARY KEY,
                collection_id TEXT NOT NULL,
                status TEXT NOT NULL,
                total_files INTEGER NOT NULL,
                processed_files INTEGER NOT NULL,
                current_file TEXT,
                error_message TEXT,
                started_at INTEGER NOT NULL,
                completed_at INTEGER
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS session_summaries (
                id TEXT PRIMARY KEY,
                org_id TEXT,
                project_id TEXT NOT NULL,
                repo_id TEXT,
                session_id TEXT NOT NULL,
                topics TEXT,
                decisions TEXT,
                next_steps TEXT,
                key_files TEXT,
                origin_context TEXT,
                created_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                checkpoint_type TEXT NOT NULL,
                description TEXT NOT NULL,
                snapshot_data TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                restored_at INTEGER,
                expired INTEGER
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS delegations (
                id TEXT PRIMARY KEY,
                parent_session_id TEXT NOT NULL,
                child_session_id TEXT NOT NULL,
                prompt TEXT NOT NULL,
                prompt_embedding_id TEXT,
                result TEXT,
                success INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                completed_at INTEGER,
                duration_ms INTEGER
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS tool_calls (
                id TEXT PRIMARY KEY,
                org_id TEXT,
                project_id TEXT,
                repo_id TEXT,
                session_id TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                params_summary TEXT,
                success INTEGER NOT NULL,
                error_message TEXT,
                duration_ms INTEGER,
                created_at INTEGER NOT NULL
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS error_patterns (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                pattern_signature TEXT NOT NULL,
                description TEXT NOT NULL,
                category TEXT NOT NULL,
                solutions TEXT,
                affected_files TEXT,
                tags TEXT,
                occurrence_count INTEGER NOT NULL,
                first_seen_at INTEGER NOT NULL,
                last_seen_at INTEGER NOT NULL,
                embedding_id TEXT
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS error_pattern_matches (
                id TEXT PRIMARY KEY,
                pattern_id TEXT NOT NULL,
                observation_id TEXT NOT NULL,
                confidence INTEGER NOT NULL,
                solution_applied INTEGER,
                resolution_successful INTEGER,
                matched_at INTEGER NOT NULL,
                resolved_at INTEGER
            )",
        )
        .await?;

        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_obs_project ON observations(project_id)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_organizations_name ON organizations(name)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_branches_repo ON branches(repository_id)",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        for trigger in ["obs_au", "obs_ad", "obs_ai"] {
            db.execute_unprepared(&format!("DROP TRIGGER IF EXISTS {trigger}"))
                .await?;
        }
        for shadow in [
            "observations_fts_config",
            "observations_fts_docsize",
            "observations_fts_content",
            "observations_fts_idx",
            "observations_fts_data",
            "observations_fts",
        ] {
            db.execute_unprepared(&format!("DROP TABLE IF EXISTS {shadow}"))
                .await?;
        }

        for table in [
            "error_pattern_matches",
            "error_patterns",
            "tool_calls",
            "delegations",
            "checkpoints",
            "session_summaries",
            "index_operations",
            "file_hashes",
            "collections",
            "observations",
            "agent_sessions",
            "issue_label_assignments",
            "issue_labels",
            "issue_comments",
            "project_decisions",
            "project_dependencies",
            "project_phases",
            "project_issues",
            "plan_reviews",
            "plan_versions",
            "plans",
            "api_keys",
            "team_members",
            "teams",
            "users",
            "agent_worktree_assignments",
            "worktrees",
            "branches",
            "repositories",
            "projects",
            "organizations",
        ] {
            db.execute_unprepared(&format!("DROP TABLE IF EXISTS {table}"))
                .await?;
        }
        Ok(())
    }
}
