use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "file_hashes",
        [
            crate::col!("id", Integer, auto),
            crate::col!("project_id", Text),
            crate::col!("collection", Text),
            crate::col!("file_path", Text),
            crate::col!("content_hash", Text),
            crate::col!("indexed_at", Integer),
            crate::col!("deleted_at", Integer, nullable),
            crate::col!("origin_context", Text, nullable),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_file_hashes_project", "file_hashes", ["project_id"]),
        crate::index!("idx_file_hashes_collection", "file_hashes", ["collection"]),
        crate::index!("idx_file_hashes_deleted", "file_hashes", ["deleted_at"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![crate::fk!("file_hashes", "project_id", "projects", "id")]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    vec![crate::unique!(
        "file_hashes",
        ["project_id", "collection", "file_path"]
    )]
}
