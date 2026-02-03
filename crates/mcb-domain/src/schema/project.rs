//! Unified project schema: all persistence entities and relationships.
//!
//! Single source of truth for the whole project: memory (observations,
//! session_summaries), collections (vector store mapping), file_hashes
//! (incremental indexing), agent sessions (agent tracking), with FKs so
//! backends generate correct REFERENCES. Aligns database port data with
//! vector stores (collections) and project org.

pub mod agent;

use super::memory::{ColumnDef, ColumnType, FtsDef, IndexDef, TableDef};

/// Foreign key: (from_table.from_column) REFERENCES to_table(to_column).
#[derive(Debug, Clone)]
pub struct ForeignKeyDef {
    pub from_table: String,
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
}

/// Composite unique constraint (e.g. UNIQUE(collection, file_path)).
#[derive(Debug, Clone)]
pub struct UniqueConstraintDef {
    pub table: String,
    pub columns: Vec<String>,
}

/// Full project schema: all tables, FTS, indexes, FKs, unique constraints.
///
/// Use this for one database that serves memory, collections, and file hashes.
/// Each backend (SQLite, PostgreSQL, MySQL) implements [`super::SchemaDdlGenerator`]
/// to produce dialect-specific DDL from this schema.
#[derive(Debug, Clone)]
pub struct ProjectSchema {
    pub tables: Vec<TableDef>,
    pub fts: Option<FtsDef>,
    pub indexes: Vec<IndexDef>,
    pub foreign_keys: Vec<ForeignKeyDef>,
    pub unique_constraints: Vec<UniqueConstraintDef>,
}

impl ProjectSchema {
    /// Returns the full project schema: collections, observations, session_summaries,
    /// file_hashes, FTS, indexes, and relationships (FKs).
    #[must_use]
    pub fn definition() -> Self {
        Self {
            tables: Self::tables(),
            fts: Self::fts_def(),
            indexes: Self::indexes(),
            foreign_keys: Self::foreign_keys(),
            unique_constraints: Self::unique_constraints(),
        }
    }

    fn tables() -> Vec<TableDef> {
        let mut tables = vec![
            TableDef {
                name: "projects".to_string(),
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
                        name: "name".to_string(),
                        type_: ColumnType::Text,
                        primary_key: false,
                        unique: true,
                        not_null: true,
                        auto_increment: false,
                    },
                    ColumnDef {
                        name: "path".to_string(),
                        type_: ColumnType::Text,
                        primary_key: false,
                        unique: false,
                        not_null: true,
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
                        name: "updated_at".to_string(),
                        type_: ColumnType::Integer,
                        primary_key: false,
                        unique: false,
                        not_null: true,
                        auto_increment: false,
                    },
                ],
            },
            // collections: user name <-> vector store name (replaces collection_mapping.json)
            TableDef {
                name: "collections".to_string(),
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
                        name: "project_id".to_string(),
                        type_: ColumnType::Text,
                        primary_key: false,
                        unique: false,
                        not_null: true,
                        auto_increment: false,
                    },
                    ColumnDef {
                        name: "name".to_string(),
                        type_: ColumnType::Text,
                        primary_key: false,
                        unique: false,
                        not_null: true,
                        auto_increment: false,
                    },
                    ColumnDef {
                        name: "vector_name".to_string(),
                        type_: ColumnType::Text,
                        primary_key: false,
                        unique: false,
                        not_null: true,
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
            // observations (memory)
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
                        name: "project_id".to_string(),
                        type_: ColumnType::Text,
                        primary_key: false,
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
                        name: "observation_type".to_string(),
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
            // session_summaries (memory)
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
                        name: "project_id".to_string(),
                        type_: ColumnType::Text,
                        primary_key: false,
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
            // file_hashes (incremental indexing; collection = namespace)
            TableDef {
                name: "file_hashes".to_string(),
                columns: vec![
                    ColumnDef {
                        name: "id".to_string(),
                        type_: ColumnType::Integer,
                        primary_key: true,
                        unique: false,
                        not_null: true,
                        auto_increment: true,
                    },
                    ColumnDef {
                        name: "project_id".to_string(),
                        type_: ColumnType::Text,
                        primary_key: false,
                        unique: false,
                        not_null: true,
                        auto_increment: false,
                    },
                    ColumnDef {
                        name: "collection".to_string(),
                        type_: ColumnType::Text,
                        primary_key: false,
                        unique: false,
                        not_null: true,
                        auto_increment: false,
                    },
                    ColumnDef {
                        name: "file_path".to_string(),
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
                        unique: false,
                        not_null: true,
                        auto_increment: false,
                    },
                    ColumnDef {
                        name: "indexed_at".to_string(),
                        type_: ColumnType::Integer,
                        primary_key: false,
                        unique: false,
                        not_null: true,
                        auto_increment: false,
                    },
                    ColumnDef {
                        name: "deleted_at".to_string(),
                        type_: ColumnType::Integer,
                        primary_key: false,
                        unique: false,
                        not_null: false,
                        auto_increment: false,
                    },
                ],
            },
        ];
        tables.extend(agent::tables());
        tables
    }

    fn fts_def() -> Option<FtsDef> {
        Some(FtsDef {
            virtual_table_name: "observations_fts".to_string(),
            content_table: "observations".to_string(),
            content_columns: vec!["content".to_string()],
            id_column: "id".to_string(),
        })
    }

    fn indexes() -> Vec<IndexDef> {
        let mut indexes = vec![
            IndexDef {
                name: "idx_collections_project".to_string(),
                table: "collections".to_string(),
                columns: vec!["project_id".to_string()],
            },
            IndexDef {
                name: "idx_obs_project".to_string(),
                table: "observations".to_string(),
                columns: vec!["project_id".to_string()],
            },
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
            IndexDef {
                name: "idx_file_hashes_project".to_string(),
                table: "file_hashes".to_string(),
                columns: vec!["project_id".to_string()],
            },
            IndexDef {
                name: "idx_file_hashes_collection".to_string(),
                table: "file_hashes".to_string(),
                columns: vec!["collection".to_string()],
            },
            IndexDef {
                name: "idx_file_hashes_deleted".to_string(),
                table: "file_hashes".to_string(),
                columns: vec!["deleted_at".to_string()],
            },
        ];
        indexes.extend(agent::indexes());
        indexes
    }

    fn foreign_keys() -> Vec<ForeignKeyDef> {
        let mut fks = vec![
            ForeignKeyDef {
                from_table: "collections".to_string(),
                from_column: "project_id".to_string(),
                to_table: "projects".to_string(),
                to_column: "id".to_string(),
            },
            ForeignKeyDef {
                from_table: "observations".to_string(),
                from_column: "project_id".to_string(),
                to_table: "projects".to_string(),
                to_column: "id".to_string(),
            },
            ForeignKeyDef {
                from_table: "session_summaries".to_string(),
                from_column: "project_id".to_string(),
                to_table: "projects".to_string(),
                to_column: "id".to_string(),
            },
            ForeignKeyDef {
                from_table: "file_hashes".to_string(),
                from_column: "project_id".to_string(),
                to_table: "projects".to_string(),
                to_column: "id".to_string(),
            },
        ];
        fks.extend(agent::foreign_keys());
        fks
    }

    fn unique_constraints() -> Vec<UniqueConstraintDef> {
        vec![
            UniqueConstraintDef {
                table: "collections".to_string(),
                columns: vec!["project_id".to_string(), "name".to_string()],
            },
            UniqueConstraintDef {
                table: "file_hashes".to_string(),
                columns: vec![
                    "project_id".to_string(),
                    "collection".to_string(),
                    "file_path".to_string(),
                ],
            },
        ]
    }
}

/// Port for generating DDL from the full project schema.
///
/// Each backend (SQLite, PostgreSQL, MySQL, etc.) implements this trait to
/// produce dialect-specific DDL for all project tables, FKs, and constraints.
/// Use this for a single database that serves memory, collections, and file hashes.
pub trait SchemaDdlGenerator: Send + Sync {
    /// Generate DDL statements for the given project schema in this backend's dialect.
    fn generate_ddl(&self, schema: &ProjectSchema) -> Vec<String>;
}
