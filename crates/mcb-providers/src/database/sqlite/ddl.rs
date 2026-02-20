//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md#database)
//!
use mcb_domain::schema::{
    ColumnType, ForeignKeyDef, FtsDef, IndexDef, Schema, SchemaDdlGenerator, TableDef,
    UniqueConstraintDef,
};

/// Generates `SQLite` DDL from the full project schema (collections, observations,
/// `session_summaries`, `file_hashes`, FTS, indexes, unique constraints).
#[derive(Debug, Clone, Default)]
pub struct SqliteSchemaDdlGenerator;

impl SchemaDdlGenerator for SqliteSchemaDdlGenerator {
    fn generate_ddl(&self, schema: &Schema) -> Vec<String> {
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
            stmts.extend(rebuild_fts_sqlite(fts));
        }
        for idx in &schema.indexes {
            stmts.push(index_to_sqlite_ddl(idx));
        }
        stmts
    }
}

fn column_type_sqlite(ty: &ColumnType) -> &'static str {
    match ty {
        ColumnType::Text | ColumnType::Json | ColumnType::Uuid => "TEXT",
        ColumnType::Integer | ColumnType::Boolean | ColumnType::Timestamp => "INTEGER",
        ColumnType::Real => "REAL",
        ColumnType::Blob => "BLOB",
    }
}

fn table_to_sqlite_ddl_with_fk(
    table: &TableDef,
    unique_constraints: &[&UniqueConstraintDef],
    foreign_keys: &[&ForeignKeyDef],
) -> String {
    let pk_cols: Vec<&str> = table
        .columns
        .iter()
        .filter(|c| c.primary_key)
        .map(|c| c.name.as_str())
        .collect();
    let is_composite_pk = pk_cols.len() > 1;

    let cols: Vec<String> = table
        .columns
        .iter()
        .map(|c| {
            let mut s = format!("{} {}", c.name, column_type_sqlite(&c.type_));
            if c.primary_key && !is_composite_pk {
                s.push_str(" PRIMARY KEY");
                if c.auto_increment && matches!(c.type_, ColumnType::Integer) {
                    s.push_str(" AUTOINCREMENT");
                }
            }
            if c.unique && !c.primary_key {
                s.push_str(" UNIQUE");
            }
            // Composite PK columns get NOT NULL (SQLite requires it explicitly
            // for composite PKs, unlike single-column PKs which imply it).
            if c.not_null && (!c.primary_key || is_composite_pk) {
                s.push_str(" NOT NULL");
            }
            if let Some(fk) = foreign_keys.iter().find(|fk| fk.from_column == c.name) {
                use std::fmt::Write;
                let _ = write!(s, " REFERENCES {}({})", fk.to_table, fk.to_column);
            }
            s
        })
        .collect();
    let mut parts = cols;
    if is_composite_pk {
        parts.push(format!("PRIMARY KEY({})", pk_cols.join(", ")));
    }
    for u in unique_constraints {
        parts.push(format!("UNIQUE({})", u.columns.join(", ")));
    }
    format!(
        "CREATE TABLE IF NOT EXISTS {} ({})",
        table.name,
        parts.join(", ")
    )
}

/// Drop stale FTS table + triggers, recreate from current schema, repopulate.
///
/// `IF NOT EXISTS` silently keeps old FTS/trigger definitions on schema
/// evolution, so we always drop and recreate to guarantee consistency.
fn rebuild_fts_sqlite(fts: &FtsDef) -> Vec<String> {
    let content_cols = fts.content_columns.join(", ");
    let content_col = fts
        .content_columns
        .first()
        .map_or("content", String::as_str);

    vec![
        "DROP TRIGGER IF EXISTS obs_ai".to_owned(),
        "DROP TRIGGER IF EXISTS obs_ad".to_owned(),
        "DROP TRIGGER IF EXISTS obs_au".to_owned(),
        format!("DROP TABLE IF EXISTS {}", fts.virtual_table_name),
        format!(
            "CREATE VIRTUAL TABLE {} USING fts5({}, {} UNINDEXED)",
            fts.virtual_table_name, content_cols, fts.id_column
        ),
        format!(
            "INSERT OR IGNORE INTO {}({}, {}) SELECT {}, {} FROM {}",
            fts.virtual_table_name,
            fts.id_column,
            content_col,
            fts.id_column,
            content_col,
            fts.content_table,
        ),
        trigger_after_insert_sqlite(fts),
        trigger_after_delete_sqlite(fts),
        trigger_after_update_sqlite(fts),
    ]
}

fn trigger_after_insert_sqlite(fts: &FtsDef) -> String {
    let content_col = fts
        .content_columns
        .first()
        .map_or("content", String::as_str);
    format!(
        "CREATE TRIGGER obs_ai AFTER INSERT ON {} BEGIN
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
        "CREATE TRIGGER obs_ad AFTER DELETE ON {} BEGIN
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
        "CREATE TRIGGER obs_au AFTER UPDATE ON {} BEGIN
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

/// Generate `ALTER TABLE ... ADD COLUMN` SQL for a single column.
///
/// `SQLite` `ADD COLUMN` requires the column to either be nullable or have a default.
/// Since our schema evolution only adds nullable columns, this is safe.
pub(crate) fn alter_table_add_column_sqlite(
    table: &str,
    col: &mcb_domain::schema::ColumnDef,
) -> String {
    let ty = column_type_sqlite(&col.type_);
    format!("ALTER TABLE {table} ADD COLUMN {} {ty}", col.name)
}
