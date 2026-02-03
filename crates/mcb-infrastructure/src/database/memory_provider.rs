//! SQLite pool provider using unified `ProjectSchema` DDL.
//!
//! Single place that opens SQLite connections and initializes the full project schema
//! (projects, collections, observations, session_summaries, file_hashes, FTS, indexes).
//! Repositories receive the pool via dependency injection and do not access SQLite directly.

use mcb_domain::error::{Error, Result};
use mcb_domain::schema::{ProjectSchema, SchemaDdlGenerator};
use sqlx::SqlitePool;
use std::path::PathBuf;
use tracing::{debug, info};

/// SQLite connection pool provider for memory storage with full `ProjectSchema` initialization.
pub struct MemoryDatabaseProvider;

struct SqliteSchemaDdlGeneratorLocal;

impl SchemaDdlGenerator for SqliteSchemaDdlGeneratorLocal {
    fn generate_ddl(&self, schema: &ProjectSchema) -> Vec<String> {
        use mcb_domain::schema::{
            ColumnType, ForeignKeyDef, FtsDef, IndexDef, TableDef, UniqueConstraintDef,
        };

        fn column_type_sqlite(ty: &ColumnType) -> &'static str {
            match ty {
                ColumnType::Text => "TEXT",
                ColumnType::Integer => "INTEGER",
            }
        }

        fn table_to_sqlite_ddl(
            table: &TableDef,
            unique_constraints: &[&UniqueConstraintDef],
            foreign_keys: &[&ForeignKeyDef],
        ) -> String {
            let cols: Vec<String> = table
                .columns
                .iter()
                .map(|c| {
                    let mut s = format!("{} {}", c.name, column_type_sqlite(&c.type_));
                    if c.primary_key {
                        s.push_str(" PRIMARY KEY");
                        if c.auto_increment && matches!(c.type_, ColumnType::Integer) {
                            s.push_str(" AUTOINCREMENT");
                        }
                    }
                    if c.unique && !c.primary_key {
                        s.push_str(" UNIQUE");
                    }
                    if c.not_null && !c.primary_key {
                        s.push_str(" NOT NULL");
                    }
                    if let Some(fk) = foreign_keys.iter().find(|fk| fk.from_column == c.name) {
                        s.push_str(&format!(" REFERENCES {}({})", fk.to_table, fk.to_column));
                    }
                    s
                })
                .collect();
            let mut parts = cols;
            for u in unique_constraints {
                parts.push(format!("UNIQUE({})", u.columns.join(", ")));
            }
            format!(
                "CREATE TABLE IF NOT EXISTS {} ({})",
                table.name,
                parts.join(", ")
            )
        }

        fn fts_to_sqlite_ddl(fts: &FtsDef) -> String {
            let content_cols = fts.content_columns.join(", ");
            format!(
                "CREATE VIRTUAL TABLE IF NOT EXISTS {} USING fts5({}, {} UNINDEXED)",
                fts.virtual_table_name, content_cols, fts.id_column
            )
        }

        fn trigger_after_insert_sqlite(fts: &FtsDef) -> String {
            let content_col = fts
                .content_columns
                .first()
                .map_or("content", String::as_str);
            format!(
                r"CREATE TRIGGER IF NOT EXISTS obs_ai AFTER INSERT ON {} BEGIN
  INSERT INTO {}({}, {}) VALUES (new.{}, new.{});
END;",
                fts.content_table,
                fts.virtual_table_name,
                fts.id_column,
                content_col,
                fts.id_column,
                content_col
            )
        }

        fn trigger_after_delete_sqlite(fts: &FtsDef) -> String {
            format!(
                r"CREATE TRIGGER IF NOT EXISTS obs_ad AFTER DELETE ON {} BEGIN
  DELETE FROM {} WHERE {} = old.{};
END;",
                fts.content_table, fts.virtual_table_name, fts.id_column, fts.id_column
            )
        }

        fn trigger_after_update_sqlite(fts: &FtsDef) -> String {
            let content_col = fts
                .content_columns
                .first()
                .map_or("content", String::as_str);
            format!(
                r"CREATE TRIGGER IF NOT EXISTS obs_au AFTER UPDATE ON {} BEGIN
  DELETE FROM {} WHERE {} = old.{};
  INSERT INTO {}({}, {}) VALUES (new.{}, new.{});
END;",
                fts.content_table,
                fts.virtual_table_name,
                fts.id_column,
                fts.id_column,
                fts.virtual_table_name,
                fts.id_column,
                content_col,
                fts.id_column,
                content_col
            )
        }

        fn index_to_sqlite_ddl(idx: &IndexDef) -> String {
            let cols = idx.columns.join(", ");
            format!(
                "CREATE INDEX IF NOT EXISTS {} ON {}({})",
                idx.name, idx.table, cols
            )
        }

        let mut stmts = Vec::new();
        for table in &schema.tables {
            let uniques: Vec<&UniqueConstraintDef> = schema
                .unique_constraints
                .iter()
                .filter(|u| u.table == table.name)
                .collect();
            let fks: Vec<&ForeignKeyDef> = schema
                .foreign_keys
                .iter()
                .filter(|fk| fk.from_table == table.name)
                .collect();
            stmts.push(table_to_sqlite_ddl(table, &uniques, &fks));
        }
        if let Some(fts) = &schema.fts {
            stmts.push(fts_to_sqlite_ddl(fts));
            stmts.push(trigger_after_insert_sqlite(fts));
            stmts.push(trigger_after_delete_sqlite(fts));
            stmts.push(trigger_after_update_sqlite(fts));
        }
        for idx in &schema.indexes {
            stmts.push(index_to_sqlite_ddl(idx));
        }
        stmts
    }
}

async fn apply_project_schema(pool: &SqlitePool) -> Result<()> {
    let generator = SqliteSchemaDdlGeneratorLocal;
    let schema = ProjectSchema::definition();
    let ddl = generator.generate_ddl(&schema);
    for sql in ddl {
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| Error::memory_with_source("apply DDL", e))?;
    }
    Ok(())
}

impl MemoryDatabaseProvider {
    pub async fn connect(path: PathBuf) -> Result<SqlitePool> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::memory_with_source("Failed to create db directory", e))?;
        }

        let db_url = format!("sqlite:{}?mode=rwc", path.display());
        let pool = SqlitePool::connect(&db_url)
            .await
            .map_err(|e| Error::memory_with_source("Failed to connect to SQLite", e))?;

        apply_project_schema(&pool).await?;

        info!("Memory database initialized at {}", path.display());
        Ok(pool)
    }

    pub async fn connect_in_memory() -> Result<SqlitePool> {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .map_err(|e| Error::memory_with_source("Failed to connect to in-memory SQLite", e))?;

        apply_project_schema(&pool).await?;

        debug!("In-memory memory database initialized");
        Ok(pool)
    }
}
