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

/// Column name for observation type in observations table (single source of truth for ORG002).
pub const COL_OBSERVATION_TYPE: &str = "observation_type";
