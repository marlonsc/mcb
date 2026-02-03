//! Database providers: memory repository and schema application per backend.
//!
//! Each backend (SQLite, PostgreSQL, MySQL, etc.) has its own submodule and
//! implements [`mcb_domain::MemorySchemaDdlGenerator`] to produce
//! dialect-specific DDL from the generic [`mcb_domain::MemorySchema`].
//!
//! - **sqlite** – SQLite adapter and DDL generator (FTS5, etc.)
//! - **postgres** – (future) PostgreSQL adapter and DDL
//! - **mysql** – (future) MySQL adapter and DDL

#[cfg(feature = "memory-sqlite")]
pub mod sqlite;
#[cfg(feature = "memory-sqlite")]
pub use sqlite::{
    SqliteDatabaseProvider, SqliteExecutor, SqliteMemoryDdlGenerator, SqliteMemoryRepository,
    SqliteSchemaDdlGenerator, create_memory_repository, create_memory_repository_in_memory,
    create_memory_repository_in_memory_with_executor,
};
