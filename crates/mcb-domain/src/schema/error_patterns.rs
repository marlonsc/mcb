use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "error_patterns",
        [
            crate::col!("id", Text, pk),
            crate::col!("project_id", Text),
            crate::col!("pattern_signature", Text),
            crate::col!("description", Text),
            crate::col!("category", Text),
            crate::col!("solutions", Text, nullable),
            crate::col!("affected_files", Text, nullable),
            crate::col!("tags", Text, nullable),
            crate::col!("occurrence_count", Integer),
            crate::col!("first_seen_at", Integer),
            crate::col!("last_seen_at", Integer),
            crate::col!("embedding_id", Text, nullable),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!(
            "idx_error_patterns_project",
            "error_patterns",
            ["project_id"]
        ),
        crate::index!(
            "idx_error_patterns_category",
            "error_patterns",
            ["category"]
        ),
        crate::index!(
            "idx_error_patterns_signature",
            "error_patterns",
            ["pattern_signature"]
        ),
        crate::index!(
            "idx_error_patterns_embedding",
            "error_patterns",
            ["embedding_id"]
        ),
        crate::index!(
            "idx_error_patterns_last_seen",
            "error_patterns",
            ["last_seen_at"]
        ),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![crate::fk!("error_patterns", "project_id", "projects", "id")]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
