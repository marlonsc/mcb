use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "plan_reviews",
        [
            crate::col!("id", Text, pk),
            crate::col!("org_id", Text),
            crate::col!("plan_version_id", Text),
            crate::col!("reviewer_id", Text),
            crate::col!("verdict", Text),
            crate::col!("feedback", Text),
            crate::col!("created_at", Integer),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!("idx_plan_reviews_org", "plan_reviews", ["org_id"]),
        crate::index!(
            "idx_plan_reviews_version",
            "plan_reviews",
            ["plan_version_id"]
        ),
        crate::index!("idx_plan_reviews_reviewer", "plan_reviews", ["reviewer_id"]),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!("plan_reviews", "org_id", "organizations", "id"),
        crate::fk!("plan_reviews", "plan_version_id", "plan_versions", "id"),
        crate::fk!("plan_reviews", "reviewer_id", "users", "id"),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
