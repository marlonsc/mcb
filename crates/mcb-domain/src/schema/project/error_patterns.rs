//! Error pattern schema elements for the project schema.

use super::ForeignKeyDef;
use crate::schema::memory::{ColumnDef, ColumnType, IndexDef, TableDef};

pub fn tables() -> Vec<TableDef> {
    vec![
        TableDef {
            name: "error_patterns".to_string(),
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
                    name: "pattern_signature".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "description".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "category".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "solutions".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: false,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "affected_files".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: false,
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
                    name: "occurrence_count".to_string(),
                    type_: ColumnType::Integer,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "first_seen_at".to_string(),
                    type_: ColumnType::Integer,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "last_seen_at".to_string(),
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
            name: "error_pattern_matches".to_string(),
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
                    name: "pattern_id".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "observation_id".to_string(),
                    type_: ColumnType::Text,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "confidence".to_string(),
                    type_: ColumnType::Integer,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "solution_applied".to_string(),
                    type_: ColumnType::Integer,
                    primary_key: false,
                    unique: false,
                    not_null: false,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "resolution_successful".to_string(),
                    type_: ColumnType::Integer,
                    primary_key: false,
                    unique: false,
                    not_null: false,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "matched_at".to_string(),
                    type_: ColumnType::Integer,
                    primary_key: false,
                    unique: false,
                    not_null: true,
                    auto_increment: false,
                },
                ColumnDef {
                    name: "resolved_at".to_string(),
                    type_: ColumnType::Integer,
                    primary_key: false,
                    unique: false,
                    not_null: false,
                    auto_increment: false,
                },
            ],
        },
    ]
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        IndexDef {
            name: "idx_error_patterns_project".to_string(),
            table: "error_patterns".to_string(),
            columns: vec!["project_id".to_string()],
        },
        IndexDef {
            name: "idx_error_patterns_category".to_string(),
            table: "error_patterns".to_string(),
            columns: vec!["category".to_string()],
        },
        IndexDef {
            name: "idx_error_patterns_last_seen".to_string(),
            table: "error_patterns".to_string(),
            columns: vec!["last_seen_at".to_string()],
        },
        IndexDef {
            name: "idx_error_pattern_matches_pattern".to_string(),
            table: "error_pattern_matches".to_string(),
            columns: vec!["pattern_id".to_string()],
        },
        IndexDef {
            name: "idx_error_pattern_matches_observation".to_string(),
            table: "error_pattern_matches".to_string(),
            columns: vec!["observation_id".to_string()],
        },
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            from_table: "error_patterns".to_string(),
            from_column: "project_id".to_string(),
            to_table: "projects".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "error_pattern_matches".to_string(),
            from_column: "pattern_id".to_string(),
            to_table: "error_patterns".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "error_pattern_matches".to_string(),
            from_column: "observation_id".to_string(),
            to_table: "observations".to_string(),
            to_column: "id".to_string(),
        },
    ]
}
