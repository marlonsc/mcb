//! SQLite-specific DDL generation from the generic schema.
//!
//! Implements [`MemorySchemaDdlGenerator`] (memory subset) and [`SchemaDdlGenerator`]
//! (full project schema) for SQLite only. Other backends (PostgreSQL, MySQL)
//! have their own modules with their own dialect.

use mcb_domain::schema::{
    ColumnType, ForeignKeyDef, FtsDef, IndexDef, MemorySchema, MemorySchemaDdlGenerator,
    ProjectSchema, SchemaDdlGenerator, TableDef, UniqueConstraintDef,
};

/// Generates SQLite DDL from the generic memory schema (observations, session_summaries, FTS).
#[derive(Debug, Clone, Default)]
pub struct SqliteMemoryDdlGenerator;

impl MemorySchemaDdlGenerator for SqliteMemoryDdlGenerator {
    fn generate_ddl(&self, schema: &MemorySchema) -> Vec<String> {
        let mut stmts = Vec::new();
        for table in &schema.tables {
            stmts.push(table_to_sqlite_ddl(table, &[]));
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

/// Generates SQLite DDL from the full project schema (collections, observations,
/// session_summaries, file_hashes, FTS, indexes, unique constraints).
#[derive(Debug, Clone, Default)]
pub struct SqliteSchemaDdlGenerator;

impl SchemaDdlGenerator for SqliteSchemaDdlGenerator {
    fn generate_ddl(&self, schema: &ProjectSchema) -> Vec<String> {
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
            stmts.push(table_to_sqlite_ddl_with_fk(table, &uniques, &fks));
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

fn column_type_sqlite(ty: &ColumnType) -> &'static str {
    match ty {
        ColumnType::Text => "TEXT",
        ColumnType::Integer => "INTEGER",
        ColumnType::Real => "REAL",
        ColumnType::Boolean => "INTEGER",
        ColumnType::Blob => "BLOB",
        ColumnType::Json => "TEXT",
        ColumnType::Uuid => "TEXT",
        ColumnType::Timestamp => "INTEGER",
    }
}

fn table_to_sqlite_ddl(table: &TableDef, unique_constraints: &[&UniqueConstraintDef]) -> String {
    table_to_sqlite_ddl_with_fk(table, unique_constraints, &[])
}

fn table_to_sqlite_ddl_with_fk(
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
