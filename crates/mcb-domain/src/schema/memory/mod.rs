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
/// Table definitions for memory entities.
pub mod tables;

pub use columns::{COL_OBSERVATION_TYPE, ColumnDef, ColumnType};
pub use fts::FtsDef;
pub use indexes::{IndexDef, indexes};
pub use tables::{TableDef, tables};

/// Complete memory schema: same model for any backend.
#[derive(Debug, Clone)]
pub struct MemorySchema {
    /// List of table definitions.
    pub tables: Vec<TableDef>,
    /// Optional full-text search definition.
    pub fts: Option<FtsDef>,
    /// List of index definitions.
    pub indexes: Vec<IndexDef>,
}

/// Port for generating DDL from the generic memory schema.
///
/// Each backend (`SQLite`, `PostgreSQL`, `MySQL`, etc.) implements this trait to
/// produce dialect-specific DDL from this model. The same [`MemorySchema`] is the single
/// source of truth; only the output format differs per backend.
///
/// # Examples
///
/// ```rust
/// use mcb_domain::schema::memory::{MemorySchema, MemorySchemaDdlGenerator};
///
/// struct SqliteGenerator;
/// impl MemorySchemaDdlGenerator for SqliteGenerator {
///     fn generate_ddl(&self, schema: &MemorySchema) -> Vec<String> {
///         vec!["CREATE TABLE ...".to_string()]
///     }
/// }
/// ```
pub trait MemorySchemaDdlGenerator: Send + Sync {
    /// Generate DDL statements for the given schema in this backend's dialect.
    ///
    /// Callers execute these statements in order to create tables, indexes,
    /// FTS, triggers, etc. on the target database.
    fn generate_ddl(&self, schema: &MemorySchema) -> Vec<String>;
}

impl MemorySchema {
    /// Returns the canonical memory schema (observations, `session_summaries`, FTS, indexes).
    #[must_use]
    pub fn definition() -> Self {
        Self {
            tables: tables(),
            fts: Some(FtsDef {
                virtual_table_name: "observations_fts".to_owned(),
                content_table: "observations".to_owned(),
                content_columns: vec!["content".to_owned()],
                id_column: "id".to_owned(),
            }),
            indexes: indexes(),
        }
    }
}
