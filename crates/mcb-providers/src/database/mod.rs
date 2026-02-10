//! Database providers: memory repository and schema application per backend.
//!
//! Each backend (SQLite, PostgreSQL, MySQL, etc.) has its own submodule and
//! implements [`mcb_domain::MemorySchemaDdlGenerator`] to produce
//! dialect-specific DDL from the generic [`mcb_domain::MemorySchema`].
//!
//! - **sqlite** – SQLite adapter and DDL generator (FTS5, etc.)
//! - **postgres** – (future) PostgreSQL adapter and DDL
//! - **mysql** – (future) MySQL adapter and DDL

pub mod sqlite;
pub use sqlite::{
    SqliteAgentRepository, SqliteDatabaseProvider, SqliteExecutor, SqliteMemoryDdlGenerator,
    SqliteMemoryRepository, SqliteProjectRepository, SqliteSchemaDdlGenerator,
    create_agent_repository, create_agent_repository_from_executor, create_memory_repository,
    create_memory_repository_with_executor, create_project_repository,
    create_project_repository_from_executor,
};
