use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "collections",
        [
            crate::col!("id", Text, pk),
            crate::col!("project_id", Text),
            crate::col!("name", Text),
            crate::col!("vector_name", Text),
            crate::col!("created_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![crate::index!(
        "idx_collections_project",
        "collections",
        ["project_id"]
    )]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![crate::fk!("collections", "project_id", "projects", "id")]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    vec![crate::unique!("collections", ["project_id", "name"])]
}
