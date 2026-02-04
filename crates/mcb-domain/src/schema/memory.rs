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
}

/// A single column definition.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub type_: ColumnType,
    pub primary_key: bool,
    pub unique: bool,
    pub not_null: bool,
    /// Auto-increment (e.g. INTEGER PRIMARY KEY AUTOINCREMENT in SQLite).
    pub auto_increment: bool,
}

/// A table definition.
#[derive(Debug, Clone)]
pub struct TableDef {
    pub name: String,
    pub columns: Vec<ColumnDef>,
}

/// Column name for observation type in observations table (single source of truth for ORG002).
pub const COL_OBSERVATION_TYPE: &str = "observation_type";

/// FTS (full-text search) definition for a content table.
#[derive(Debug, Clone)]
pub struct FtsDef {
    pub virtual_table_name: String,
    pub content_table: String,
    pub content_columns: Vec<String>,
    pub id_column: String,
}

/// Index definition.
#[derive(Debug, Clone)]
pub struct IndexDef {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
}

/// Complete memory schema: same model for any backend.
#[derive(Debug, Clone)]
pub struct MemorySchema {
    pub tables: Vec<TableDef>,
    pub fts: Option<FtsDef>,
    pub indexes: Vec<IndexDef>,
}

/// Port for generating DDL from the generic memory schema.
///
/// Each backend (SQLite, PostgreSQL, MySQL, etc.) implements this trait to
/// produce dialect-specific DDL. The same [`MemorySchema`] is the single
/// source of truth; only the output format differs per backend.
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
            tables: vec![
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
            ],
            fts: Some(FtsDef {
                virtual_table_name: "observations_fts".to_string(),
                content_table: "observations".to_string(),
                content_columns: vec!["content".to_string()],
                id_column: "id".to_string(),
            }),
            indexes: vec![
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
            ],
        }
    }
}
