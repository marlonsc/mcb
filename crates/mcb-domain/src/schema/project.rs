//! Unified project schema: all persistence entities and relationships.
//!
//! Single source of truth for the whole project: memory (observations,
//! session_summaries), collections (vector store mapping), file_hashes
//! (incremental indexing), agent sessions (agent tracking), with FKs so
//! backends generate correct REFERENCES. Aligns database port data with
//! vector stores (collections) and project org.

pub mod agent;
pub mod error_patterns;
pub mod issue_entities;
pub mod multi_tenant;
pub mod plan_entities;
pub mod vcs_entities;

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
        let memory_tables = super::memory::tables().into_iter().map(|mut t| {
            // Add project_id column if not present (memory schema might be generic)
            // In memory.rs refactor, we saw memory.rs has specific columns.
            // Let's assume memory.rs definition is correct but might lack project_id if it was designed to be generic?
            // Wait, in previous read of project.rs, 'observations' HAD project_id.
            // In memory.rs, it DID NOT have project_id in my last read.
            // So project.rs extends memory tables with project_id?
            // Or I should add project_id to memory.rs if it's supposed to be there.
            // If I blindly use memory::tables(), I might miss project_id.

            // Let's check memory.rs content again in tool 219 output.
            // It does NOT have project_id.
            // So I need to inject project_id into the tables from memory.rs.

            // Only inject if not present
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
        indexes.extend(super::memory::indexes());

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
