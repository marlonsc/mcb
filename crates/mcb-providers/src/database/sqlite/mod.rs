//! SQLite backend for the memory and full project schema.
//!
//! Implements [`MemorySchemaDdlGenerator`](mcb_domain::MemorySchemaDdlGenerator) (memory subset)
//! and [`SchemaDdlGenerator`](mcb_domain::schema::SchemaDdlGenerator) (full project: collections,
//! observations, session_summaries, file_hashes) for SQLite.
//! Provides [`SqliteExecutor`] (port `DatabaseExecutor`), [`SqliteMemoryRepository`]
//! (port `MemoryRepository`), and factory functions for DI.

mod agent_repository;
mod ddl;
pub(crate) mod ensure_parent;
pub mod executor;
mod file_hash_repository;
mod issue_entity_repository;
mod memory_repository;
mod org_entity_repository;
mod plan_entity_repository;
mod project_repository;
mod provider;
mod query_helpers;
mod row_convert;
/// Shared SQLite row mapping and extraction helper functions.
pub mod row_helpers;
mod vcs_entity_repository;

pub use agent_repository::SqliteAgentRepository;
pub use ddl::{SqliteMemoryDdlGenerator, SqliteSchemaDdlGenerator};
pub use executor::SqliteExecutor;
pub use file_hash_repository::{SqliteFileHashConfig, SqliteFileHashRepository};
pub use issue_entity_repository::SqliteIssueEntityRepository;
pub use memory_repository::SqliteMemoryRepository;
pub use org_entity_repository::SqliteOrgEntityRepository;
pub use plan_entity_repository::SqlitePlanEntityRepository;
pub use project_repository::SqliteProjectRepository;
pub use provider::*;
pub use vcs_entity_repository::SqliteVcsEntityRepository;
