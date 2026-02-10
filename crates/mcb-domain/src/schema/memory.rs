//! Generic schema for the memory model (observations, session summaries, FTS).
//!
//! Single source of truth for the persistence shape. Each backend (SQLite,
//! PostgreSQL, MySQL, etc.) implements [`MemorySchemaDdlGenerator`] to
//! produce dialect-specific DDL from this model.

/// Column type in the generic schema (no SQL dialect).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    /// Variable-length text
    Text,
    /// 64-bit integer
    Integer,
    /// Floating-point number
    Real,
    /// Boolean (stored as INTEGER 0/1 in SQLite)
    Boolean,
    /// Binary large object
    Blob,
    /// JSON data (stored as TEXT in SQLite, JSONB in Postgres)
    Json,
    /// UUID (stored as TEXT in SQLite, UUID in Postgres)
    Uuid,
    /// Timestamp (stored as INTEGER in SQLite, TIMESTAMPTZ in Postgres)
    Timestamp,
}

/// A single column definition.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// Column name.
    pub name: String,
    /// Column data type.
    pub type_: ColumnType,
    /// Whether this column is a primary key.
    pub primary_key: bool,
    /// Whether this column has a unique constraint.
    pub unique: bool,
    /// Whether this column disallows NULL values.
    pub not_null: bool,
    /// Auto-increment (e.g. INTEGER PRIMARY KEY AUTOINCREMENT in SQLite).
    pub auto_increment: bool,
}

/// A table definition.
#[derive(Debug, Clone)]
pub struct TableDef {
    /// Table name.
    pub name: String,
    /// List of column definitions for this table.
    pub columns: Vec<ColumnDef>,
}

/// Column name for observation type in observations table (single source of truth for ORG002).
pub const COL_OBSERVATION_TYPE: &str = "observation_type";

/// FTS (full-text search) definition for a content table.
#[derive(Debug, Clone)]
pub struct FtsDef {
    /// Name of the virtual FTS table.
    pub virtual_table_name: String,
    /// Name of the content table being indexed.
    pub content_table: String,
    /// Columns from the content table to include in the FTS index.
    pub content_columns: Vec<String>,
    /// Column name that uniquely identifies rows in the content table.
    pub id_column: String,
}

/// Index definition.
#[derive(Debug, Clone)]
pub struct IndexDef {
    /// Index name.
    pub name: String,
    /// Table name this index is on.
    pub table: String,
    /// Columns included in this index.
    pub columns: Vec<String>,
}

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
/// Each backend (SQLite, PostgreSQL, MySQL, etc.) implements this trait to
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
    /// Returns the canonical memory schema (observations, session_summaries, FTS, indexes).
    #[must_use]
    pub fn definition() -> Self {
        Self {
            tables: tables(),
            fts: Some(FtsDef {
                virtual_table_name: "observations_fts".to_string(),
                content_table: "observations".to_string(),
                content_columns: vec!["content".to_string()],
                id_column: "id".to_string(),
            }),
            indexes: indexes(),
        }
    }
}

/// Returns table definitions for the memory module.
pub fn tables() -> Vec<TableDef> {
    vec![
        TableDef {
            name: "observations".to_string(),
            columns: vec![
                ColumnDef {
                    name: "id".to_string(),
                    type_: ColumnType::Text,
                    primary_key: true,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "content".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "content_hash".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: true,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "tags".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: false,
                    auto_increment: false,
                },
                ColumnDef {
                    name: COL_OBSERVATION_TYPE.to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: false,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "metadata".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: false,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "created_at".to_string(),
                    type_: ColumnType::Integer,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "embedding_id".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: false,
                    auto_increment: false,
                },
            ],
        },
        TableDef {
            name: "session_summaries".to_string(),
            columns: vec![
                ColumnDef {
                    name: "id".to_string(),
                    type_: ColumnType::Text,
                    primary_key: true,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "session_id".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "topics".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: false,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "decisions".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: false,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "next_steps".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: false,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "key_files".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: false,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "created_at".to_string(),
                    type_: ColumnType::Integer,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
            ],
        },
    ]
}

/// Returns index definitions for the memory module.
pub fn indexes() -> Vec<IndexDef> {
    vec![
        IndexDef {
            name: "idx_obs_hash".to_string(),
            table: "observations".to_string(),
            columns: vec!["content_hash".to_string()],
        },
        IndexDef {
            name: "idx_obs_created".to_string(),
            table: "observations".to_string(),
            columns: vec!["created_at".to_string()],
        },
        IndexDef {
            name: "idx_summary_session".to_string(),
            table: "session_summaries".to_string(),
            columns: vec!["session_id".to_string()],
        },
    ]
}
