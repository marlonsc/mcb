use crate::schema::types::{ForeignKeyDef, IndexDef, TableDef, UniqueConstraintDef};

pub fn table() -> TableDef {
    crate::table!(
        "error_pattern_matches",
        [
            crate::col!("id", Text, pk),
            crate::col!("pattern_id", Text),
            crate::col!("observation_id", Text),
            crate::col!("confidence", Integer),
            crate::col!("solution_applied", Integer, nullable),
            crate::col!("resolution_successful", Integer, nullable),
            crate::col!("matched_at", Integer),
            crate::col!("resolved_at", Integer, nullable),
        ]
    )
}

pub fn indexes() -> Vec<IndexDef> {
    vec![
        crate::index!(
            "idx_error_pattern_matches_pattern",
            "error_pattern_matches",
            ["pattern_id"]
        ),
        crate::index!(
            "idx_error_pattern_matches_observation",
            "error_pattern_matches",
            ["observation_id"]
        ),
    ]
}

pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        crate::fk!(
            "error_pattern_matches",
            "pattern_id",
            "error_patterns",
            "id"
        ),
        crate::fk!(
            "error_pattern_matches",
            "observation_id",
            "observations",
            "id"
        ),
    ]
}

pub fn unique_constraints() -> Vec<UniqueConstraintDef> {
    Vec::new()
}
