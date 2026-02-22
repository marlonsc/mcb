//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::schema::types::{
    ColumnDef, ColumnType, ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef,
};

pub fn table() -> TableDef {
    TableDef {
        name: "checkpoints".to_owned(),
        columns: vec![
            ColumnDef {
                name: "id".to_owned(),
                type_: ColumnType::Text,
                primary_key: true,
                unique: true,
                not_null: true,
                auto_increment: false,
            },
            ColumnDef {
                name: "session_id".to_owned(),
                type_: ColumnType::Text,
                primary_key: false,
                unique: false,
                not_null: true,
                auto_increment: false,
            },
            ColumnDef {
                name: "checkpoint_type".to_owned(),
                type_: ColumnType::Text,
                primary_key: false,
                unique: false,
                not_null: true,
                auto_increment: false,
            },
            ColumnDef {
                name: "description".to_owned(),
                type_: ColumnType::Text,
                primary_key: false,
                unique: false,
                not_null: true,
                auto_increment: false,
            },
            ColumnDef {
                name: "snapshot_data".to_owned(),
                type_: ColumnType::Text,
                primary_key: false,
                unique: false,
                not_null: true,
                auto_increment: false,
            },
            ColumnDef {
                name: "created_at".to_owned(),
                type_: ColumnType::Integer,
                primary_key: false,
                unique: false,
                not_null: true,
                auto_increment: false,
            },
            ColumnDef {
                name: "restored_at".to_owned(),
                type_: ColumnType::Integer,
                primary_key: false,
                unique: false,
                not_null: false,
                auto_increment: false,
            },
            ColumnDef {
                name: "expired".to_owned(),
                type_: ColumnType::Integer,
                primary_key: false,
                unique: false,
                not_null: false,
                auto_increment: false,
            },
        ],
    }
}

pub fn indexes() -> Vec<IndexDef> {
    vec![crate::index!(
        "idx_checkpoints_session",
        "checkpoints",
        ["session_id"]
    )]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![crate::fk!(
        "checkpoints",
        "session_id",
        "agent_sessions",
        "id"
    )]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
