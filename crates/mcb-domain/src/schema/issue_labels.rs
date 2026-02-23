//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "issue_labels",
        [
            crate::col!("id", Text, pk),
            crate::col!("org_id", Text),
            crate::col!("project_id", Text),
            crate::col!("name", Text),
            crate::col!("color", Text),
            crate::col!("created_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_issue_labels_org", "issue_labels", ["org_id"]),
        crate::index!("idx_issue_labels_project", "issue_labels", ["project_id"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!("issue_labels", "org_id", "organizations", "id"),
        crate::fk!("issue_labels", "project_id", "projects", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    vec![crate::unique!(
        "issue_labels",
        ["org_id", "project_id", "name"]
    )]
}
