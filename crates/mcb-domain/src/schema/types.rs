//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
/// Logical column type used by the canonical schema model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    /// UTF-8 text value.
    Text,
    /// Signed integer value.
    Integer,
    /// Floating-point value.
    Real,
    /// Boolean represented by backend-specific primitive.
    Boolean,
    /// Arbitrary binary payload.
    Blob,
    /// JSON-encoded text value.
    Json,
    /// UUID stored in backend-supported format.
    Uuid,
    /// Unix timestamp value.
    Timestamp,
}

/// Canonical column definition.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// Column name.
    pub name: String,
    /// Logical value type.
    pub type_: ColumnType,
    /// Whether this column is part of the primary key.
    pub primary_key: bool,
    /// Whether this column has a unique constraint.
    pub unique: bool,
    /// Whether this column is non-nullable.
    pub not_null: bool,
    /// Whether this column auto-increments.
    pub auto_increment: bool,
}

/// Canonical table definition.
#[derive(Debug, Clone)]
pub struct TableDef {
    /// Table name.
    pub name: String,
    /// Ordered list of columns.
    pub columns: Vec<ColumnDef>,
}

/// Canonical secondary index definition.
#[derive(Debug, Clone)]
pub struct IndexDef {
    /// Index name.
    pub name: String,
    /// Table that owns the index.
    pub table: String,
    /// Ordered indexed columns.
    pub columns: Vec<String>,
}

/// Full-text search virtual table definition.
#[derive(Debug, Clone)]
pub struct FtsDef {
    /// Virtual table name.
    pub virtual_table_name: String,
    /// Content table used by the virtual table.
    pub content_table: String,
    /// Content columns indexed by FTS.
    pub content_columns: Vec<String>,
    /// Primary key column mirrored into the virtual table.
    pub id_column: String,
}

/// Foreign key relationship definition.
#[derive(Debug, Clone)]
pub struct ForeignKeyDef {
    /// Source table.
    pub from_table: String,
    /// Source column.
    pub from_column: String,
    /// Referenced table.
    pub to_table: String,
    /// Referenced column.
    pub to_column: String,
}

/// Multi-column uniqueness constraint definition.
#[derive(Debug, Clone)]
pub struct UniqueConstraintDef {
    /// Table that owns the constraint.
    pub table: String,
    /// Ordered columns in the unique set.
    pub columns: Vec<String>,
}

/// Column name for observation type in observations table (single source of truth for ORG002).
pub const COL_OBSERVATION_TYPE: &str = "observation_type";

/// Canonical database schema: all tables, FTS, indexes, FKs, unique constraints.
///
/// Single source of truth for the full persistence model.
/// Each backend (`SQLite`, `PostgreSQL`, `MySQL`) implements [`SchemaDdlGenerator`]
/// to produce dialect-specific DDL from this schema.
#[derive(Debug, Clone)]
pub struct Schema {
    /// All physical tables in canonical creation order.
    pub tables: Vec<TableDef>,
    /// Optional full-text search definition.
    pub fts: Option<FtsDef>,
    /// Secondary indexes across all tables.
    pub indexes: Vec<IndexDef>,
    /// Foreign key relationships.
    pub foreign_keys: Vec<ForeignKeyDef>,
    /// Uniqueness constraints.
    pub unique_constraints: Vec<UniqueConstraintDef>,
}

/// Port for generating DDL from the canonical schema.
pub trait SchemaDdlGenerator: Send + Sync {
    /// Generates backend-specific DDL statements from the canonical schema.
    fn generate_ddl(&self, schema: &Schema) -> Vec<String>;
}
