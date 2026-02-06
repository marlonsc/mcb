//! SQLite backend for the memory and full project schema.
//!
//! Implements [`MemorySchemaDdlGenerator`](mcb_domain::MemorySchemaDdlGenerator) (memory subset)
//! and [`SchemaDdlGenerator`](mcb_domain::schema::SchemaDdlGenerator) (full project: collections,
//! observations, session_summaries, file_hashes) for SQLite.
//! Provides [`SqliteExecutor`] (port [`DatabaseExecutor`]), [`SqliteMemoryRepository`]
//! (port [`MemoryRepository`]), and factory functions for DI.

mod agent_repository;
mod ddl;
mod executor;
mod memory_repository;
mod project_repository;
mod provider;
mod row_convert;

pub use agent_repository::SqliteAgentRepository;
pub use ddl::{SqliteMemoryDdlGenerator, SqliteSchemaDdlGenerator};
pub use executor::SqliteExecutor;
pub use memory_repository::SqliteMemoryRepository;
pub use project_repository::SqliteProjectRepository;
pub use provider::*;
