use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "issue_label_assignments",
        [
            crate::col!("issue_id", Text, pk),
            crate::col!("label_id", Text, pk),
            crate::col!("created_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!(
            "idx_issue_label_assignments_issue",
            "issue_label_assignments",
            ["issue_id"]
        ),
        crate::index!(
            "idx_issue_label_assignments_label",
            "issue_label_assignments",
            ["label_id"]
        ),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!(
            "issue_label_assignments",
            "issue_id",
            "project_issues",
            "id"
        ),
        crate::fk!("issue_label_assignments", "label_id", "issue_labels", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    vec![crate::unique!(
        "issue_label_assignments",
        ["issue_id", "label_id"]
    )]
}
