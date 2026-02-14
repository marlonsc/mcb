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
