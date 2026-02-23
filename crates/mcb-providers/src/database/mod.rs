//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#database)
//!
//! Database providers: memory repository and schema application per backend.
//!
//! Each backend (SQLite, PostgreSQL, MySQL, etc.) has its own submodule and
//! implements [`mcb_domain::schema::SchemaDdlGenerator`] to produce
//! dialect-specific DDL from the generic [`mcb_domain::schema::Schema`].
//!
//! - **sqlite** – SQLite adapter and DDL generator (FTS5, etc.)
//! - **postgres** – (future) PostgreSQL adapter and DDL
//! - **mysql** – (future) MySQL adapter and DDL

pub mod seaorm;
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
