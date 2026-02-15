//! Unified project schema types.

use super::super::memory::{ColumnDef, ColumnType, FtsDef, IndexDef, TableDef};

use super::agent;
use super::error_patterns;
use super::issue_entities;
use super::multi_tenant;
use super::plan_entities;
use super::vcs_entities;

/// Foreign key: (from_table.from_column) REFERENCES to_table(to_column).
#[derive(Debug, Clone)]
pub struct ForeignKeyDef {
    /// Stores the from table value.
    pub from_table: String,
    /// Stores the from column value.
    pub from_column: String,
    /// Stores the to table value.
    pub to_table: String,
    /// Stores the to column value.
    pub to_column: String,
}

/// Composite unique constraint (e.g. UNIQUE(collection, file_path)).
#[derive(Debug, Clone)]
pub struct UniqueConstraintDef {
    /// Stores the table value.
    pub table: String,
    /// Stores the columns value.
    pub columns: Vec<String>,
}

/// Full project schema: all tables, FTS, indexes, FKs, unique constraints.
///
/// Use this for one database that serves memory, collections, and file hashes.
/// Each backend (SQLite, PostgreSQL, MySQL) implements [`SchemaDdlGenerator`]
/// to produce dialect-specific DDL from this schema.
#[derive(Debug, Clone)]
pub struct ProjectSchema {
    /// Stores the tables value.
    pub tables: Vec<TableDef>,
    /// Stores the fts value.
    pub fts: Option<FtsDef>,
    /// Stores the indexes value.
    pub indexes: Vec<IndexDef>,
    /// Stores the foreign keys value.
    pub foreign_keys: Vec<ForeignKeyDef>,
    /// Stores the unique constraints value.
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

    /// Returns the table definitions for the project schema.
    fn tables() -> Vec<TableDef> {
        // Multi-tenant tables must come first (organizations referenced by projects.org_id)
        let mut tables = multi_tenant::tables();

        tables.extend(vec![
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
                        name: "org_id".to_string(),
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
        ]);

        // Add memory tables (observations, session_summaries)
        let memory_tables = super::super::memory::tables().into_iter().map(|mut t| {
            if !t.columns.iter().any(|c| c.name == "project_id") {
                t.columns.insert(
                    1,
                    ColumnDef {
                        // Insert after id
                        name: "project_id".to_string(),
                        type_: ColumnType::Text,
                        primary_key: false,
                        unique: false,
                        not_null: true,
                        auto_increment: false,
                    },
                );
            }
            t
        });

        tables.extend(memory_tables);

        tables.push(
            // file_hashes (incremental indexing; collection = namespace)
            TableDef {
                name: "file_hashes".to_string(),
                // TODO(qlty): Found 66 lines of similar code in 2 locations (mass = 145)
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
                    ColumnDef {
                        name: "origin_context".to_string(),
                        type_: ColumnType::Text,
                        primary_key: false,
                        unique: false,
                        not_null: false,
                        auto_increment: false,
                    },
                ],
            },
        );

        tables.extend(agent::tables());
        tables.extend(error_patterns::tables());
        tables.extend(issue_entities::tables());
        tables.extend(plan_entities::tables());
        tables.extend(vcs_entities::tables());
        tables
    }
}

impl ProjectSchema {
    /// Returns the FTS definition.
    fn fts_def() -> Option<FtsDef> {
        Some(FtsDef {
            virtual_table_name: "observations_fts".to_string(),
            content_table: "observations".to_string(),
            content_columns: vec!["content".to_string()],
            id_column: "id".to_string(),
        })
    }

    /// Returns the index definitions.
    fn indexes() -> Vec<IndexDef> {
        let mut indexes = vec![
            IndexDef {
                name: "idx_projects_org".to_string(),
                table: "projects".to_string(),
                columns: vec!["org_id".to_string()],
            },
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
                name: "idx_summary_project".to_string(),
                table: "session_summaries".to_string(),
                columns: vec!["project_id".to_string()],
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

        // Add memory indexes
        indexes.extend(super::super::memory::indexes());

        indexes.extend(agent::indexes());
        indexes.extend(error_patterns::indexes());
        indexes.extend(issue_entities::indexes());
        indexes.extend(multi_tenant::indexes());
        indexes.extend(plan_entities::indexes());
        indexes.extend(vcs_entities::indexes());
        indexes
    }
}

impl ProjectSchema {
    /// Returns the foreign key definitions.
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
        fks.extend(error_patterns::foreign_keys());
        fks.extend(issue_entities::foreign_keys());
        fks.extend(multi_tenant::foreign_keys());
        fks.extend(plan_entities::foreign_keys());
        fks.extend(vcs_entities::foreign_keys());
        fks
    }

    /// Returns the unique constraint definitions.
    fn unique_constraints() -> Vec<UniqueConstraintDef> {
        let mut ucs = vec![
            UniqueConstraintDef {
                table: "projects".to_string(),
                columns: vec!["org_id".to_string(), "name".to_string()],
            },
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
        ];
        ucs.extend(multi_tenant::unique_constraints());
        ucs.extend(issue_entities::unique_constraints());
        ucs.extend(plan_entities::unique_constraints());
        ucs.extend(vcs_entities::unique_constraints());
        ucs
    }
}

/// Port for generating DDL from the full project schema.
///
/// Each backend (SQLite, PostgreSQL, MySQL, etc.) implements this trait to
/// produce dialect-specific DDL for all project tables, FKs, and constraints.
/// Use this for a single database that serves memory, collections, and file hashes.
///
/// # Examples
///
/// ```rust
/// use mcb_domain::schema::project::{ProjectSchema, SchemaDdlGenerator};
///
/// struct PostgresGenerator;
/// impl SchemaDdlGenerator for PostgresGenerator {
///     fn generate_ddl(&self, schema: &ProjectSchema) -> Vec<String> {
///         vec!["CREATE TABLE ...".to_string()]
///     }
/// }
/// ```
pub trait SchemaDdlGenerator: Send + Sync {
    /// Generate DDL statements for the given project schema in this backend's dialect.
    fn generate_ddl(&self, schema: &ProjectSchema) -> Vec<String>;
}
