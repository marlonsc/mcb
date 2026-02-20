//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "issue_comments",
        [
            crate::col!("id", Text, pk),
            crate::col!("issue_id", Text),
            crate::col!("author_id", Text),
            crate::col!("content", Text),
            crate::col!("created_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_issue_comments_issue", "issue_comments", ["issue_id"]),
        crate::index!("idx_issue_comments_author", "issue_comments", ["author_id"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!("issue_comments", "issue_id", "project_issues", "id"),
        crate::fk!("issue_comments", "author_id", "users", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
