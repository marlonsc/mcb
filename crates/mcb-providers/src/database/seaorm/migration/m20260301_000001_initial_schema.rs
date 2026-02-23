use sea_orm_migration::prelude::*;

use mcb_domain::schema::{Schema, SchemaDdlGenerator};

use crate::database::sqlite::SqliteSchemaDdlGenerator;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("PRAGMA foreign_keys = ON").await?;

        let schema = Schema::definition();
        let generator = SqliteSchemaDdlGenerator;
        let stmts = generator.generate_ddl(&schema);

        for stmt in &stmts {
            db.execute_unprepared(stmt).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("DROP TRIGGER IF EXISTS obs_ai")
            .await?;
        db.execute_unprepared("DROP TRIGGER IF EXISTS obs_ad")
            .await?;
        db.execute_unprepared("DROP TRIGGER IF EXISTS obs_au")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS observations_fts")
            .await?;

        let tables = [
            "agent_worktree_assignments",
            "worktrees",
            "branches",
            "repositories",
            "plan_reviews",
            "plan_versions",
            "plans",
            "issue_label_assignments",
            "issue_labels",
            "issue_comments",
            "project_issues",
            "error_pattern_matches",
            "error_patterns",
            "checkpoints",
            "tool_calls",
            "delegations",
            "agent_sessions",
            "file_hashes",
            "session_summaries",
            "observations",
            "collections",
            "projects",
            "api_keys",
            "team_members",
            "teams",
            "users",
            "organizations",
        ];

        for table in tables {
            db.execute_unprepared(&format!("DROP TABLE IF EXISTS {table}"))
                .await?;
        }

        Ok(())
    }
}
