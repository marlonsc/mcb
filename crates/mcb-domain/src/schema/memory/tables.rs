use super::columns::{COL_OBSERVATION_TYPE, ColumnDef, ColumnType};

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
                    name: COL_OBSERVATION_TYPE.to_string(),
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
        TableDef {
            name: "session_summaries".to_string(),
            // TODO(qlty): Found 66 lines of similar code in 2 locations (mass = 145)
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
                    name: "origin_context".to_string(),
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
    ]
}
