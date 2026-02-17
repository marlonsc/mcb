//! Database providers: memory repository and schema application per backend.
//!
//! Each backend (SQLite, PostgreSQL, MySQL, etc.) has its own submodule and
//! implements [`mcb_domain::SchemaDdlGenerator`] to produce
//! dialect-specific DDL from the generic [`mcb_domain::Schema`].
//!
//! - **sqlite** – SQLite adapter and DDL generator (FTS5, etc.)
//! - **postgres** – (future) PostgreSQL adapter and DDL
//! - **mysql** – (future) MySQL adapter and DDL

pub mod sqlite;
pub use sqlite::{
    SqliteAgentRepository, SqliteDatabaseProvider, SqliteExecutor, SqliteFileHashConfig,
    SqliteFileHashRepository, SqliteIssueEntityRepository, SqliteMemoryRepository,
    SqliteOrgEntityRepository, SqlitePlanEntityRepository, SqliteProjectRepository,
    SqliteSchemaDdlGenerator, SqliteVcsEntityRepository, create_agent_repository,
    create_agent_repository_from_executor, create_memory_repository,
    create_memory_repository_with_executor, create_project_repository,
    create_project_repository_from_executor, create_vcs_entity_repository_from_executor,
};
