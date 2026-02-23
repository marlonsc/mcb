//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "index_operations",
        [
            crate::col!("id", Text, pk),
            crate::col!("collection_id", Text),
            crate::col!("status", Text),
            crate::col!("total_files", Integer),
            crate::col!("processed_files", Integer),
            crate::col!("current_file", Text, nullable),
            crate::col!("error_message", Text, nullable),
            crate::col!("started_at", Integer),
            crate::col!("completed_at", Integer, nullable),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!(
            "idx_index_operations_collection",
            "index_operations",
            ["collection_id"]
        ),
        crate::index!(
            "idx_index_operations_status",
            "index_operations",
            ["status"]
        ),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    vec![]
}
