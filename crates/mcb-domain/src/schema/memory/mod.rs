//! Generic schema for the memory model (observations, `session_summaries`, FTS).
//!
//! Single source of truth for the persistence shape. Each backend (`SQLite`,
//! `PostgreSQL`, `MySQL`, etc.) implements [`MemorySchemaDdlGenerator`] to
//! produce dialect-specific DDL from this model.
//!
//! Refactored to separate concerns (SRP).

/// Column definitions for memory tables.
pub mod columns;
/// Full-text search schema definitions.
pub mod fts;
/// Secondary index schema definitions.
pub mod indexes;
/// Complete memory schema implementation.
pub mod schema;
/// Table definitions for memory entities.
pub mod tables;

pub use columns::{COL_OBSERVATION_TYPE, ColumnDef, ColumnType};
pub use fts::FtsDef;
pub use indexes::{IndexDef, indexes};
pub use schema::{MemorySchema, MemorySchemaDdlGenerator};
pub use tables::{TableDef, tables};
