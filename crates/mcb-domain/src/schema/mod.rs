//! Generic schema definitions for persistence models.
//!
//! Describes tables, columns, indexes, FKs, and FTS in a database-agnostic way so
//! that multiple backends (SQLite, PostgreSQL, MySQL, etc.) can apply the same
//! model. Use [`ProjectSchema`] for the full project (memory + collections + file_hashes);
//! each backend implements [`SchemaDdlGenerator`] to produce dialect-specific DDL.

pub mod memory;
pub mod project;

pub use memory::{
    ColumnDef, ColumnType, FtsDef, IndexDef, MemorySchema, MemorySchemaDdlGenerator, TableDef,
    COL_OBSERVATION_TYPE,
};
pub use project::{ForeignKeyDef, ProjectSchema, SchemaDdlGenerator, UniqueConstraintDef};
