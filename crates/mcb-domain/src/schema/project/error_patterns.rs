//! Error pattern schema elements for the project schema.

use super::ForeignKeyDef;
use crate::schema::memory::{IndexDef, TableDef};

/// Returns the table definitions.
pub fn tables() -> Vec<TableDef> {
    vec![
        table!(
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
        ),
        table!(
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
        ),
    ]
}

/// Returns the index definitions.
pub fn indexes() -> Vec<IndexDef> {
    vec![
        index!(
            "idx_error_patterns_project",
            "error_patterns",
            ["project_id"]
        ),
        index!(
            "idx_error_patterns_category",
            "error_patterns",
            ["category"]
        ),
        index!(
            "idx_error_patterns_last_seen",
            "error_patterns",
            ["last_seen_at"]
        ),
        index!(
            "idx_error_pattern_matches_pattern",
            "error_pattern_matches",
            ["pattern_id"]
        ),
        index!(
            "idx_error_pattern_matches_observation",
            "error_pattern_matches",
            ["observation_id"]
        ),
    ]
}

/// Returns the foreign key definitions.
pub fn foreign_keys() -> Vec<ForeignKeyDef> {
    vec![
        ForeignKeyDef {
            from_table: "error_patterns".to_string(),
            from_column: "project_id".to_string(),
            to_table: "projects".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "error_pattern_matches".to_string(),
            from_column: "pattern_id".to_string(),
            to_table: "error_patterns".to_string(),
            to_column: "id".to_string(),
        },
        ForeignKeyDef {
            from_table: "error_pattern_matches".to_string(),
            from_column: "observation_id".to_string(),
            to_table: "observations".to_string(),
            to_column: "id".to_string(),
        },
    ]
}
