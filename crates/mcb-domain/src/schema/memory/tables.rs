use super::columns::{COL_OBSERVATION_TYPE, ColumnDef};

/// A table definition.
#[derive(Debug, Clone)]
pub struct TableDef {
    /// Table name.
    pub name: String,
    /// List of column definitions for this table.
    pub columns: Vec<ColumnDef>,
}

/// Returns the table definitions (observations, session_summaries) for the memory module.
pub fn tables() -> Vec<TableDef> {
    vec![
        crate::table!(
            "observations",
            [
                crate::col!("id", Text, pk),
                crate::col!("content", Text),
                crate::col!("content_hash", Text, unique),
                crate::col!("tags", Text, nullable),
                crate::col!(COL_OBSERVATION_TYPE, Text, nullable),
                crate::col!("metadata", Text, nullable),
                crate::col!("created_at", Integer),
                crate::col!("embedding_id", Text, nullable),
            ]
        ),
        crate::table!(
            "session_summaries",
            [
                crate::col!("id", Text, pk),
                crate::col!("session_id", Text),
                crate::col!("topics", Text, nullable),
                crate::col!("decisions", Text, nullable),
                crate::col!("next_steps", Text, nullable),
                crate::col!("key_files", Text, nullable),
                crate::col!("origin_context", Text, nullable),
                crate::col!("created_at", Integer),
            ]
        ),
    ]
}
