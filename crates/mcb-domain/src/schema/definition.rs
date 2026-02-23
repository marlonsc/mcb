//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use super::types::{
    ForeignKeyDef, FtsDef, HasTableSchema, IndexDef, Schema, TableDef, UniqueConstraintDef,
};

// Schema modules not yet migrated to HasTableSchema
use super::{
    agent_sessions, agent_worktree_assignments, branches, checkpoints, collections, delegations,
    error_pattern_matches, error_patterns, file_hashes, issue_comments, issue_label_assignments,
    issue_labels, observations, plan_reviews, plan_versions, plans, project_issues, projects,
    repositories, session_summaries, tool_calls, worktrees,
};

struct SchemaEntry {
    table: fn() -> TableDef,
    indexes: fn() -> Vec<IndexDef>,
    foreign_keys: fn() -> Vec<ForeignKeyDef>,
    unique_constraints: fn() -> Vec<UniqueConstraintDef>,
}

/// Build a [`SchemaEntry`] from a type implementing [`HasTableSchema`].
macro_rules! from_entity {
    ($entity:ty) => {
        SchemaEntry {
            table: <$entity as HasTableSchema>::table_def,
            indexes: <$entity as HasTableSchema>::indexes,
            foreign_keys: <$entity as HasTableSchema>::foreign_keys,
            unique_constraints: <$entity as HasTableSchema>::unique_constraints,
        }
    };
}

/// Build a [`SchemaEntry`] from a legacy schema module (4 free functions).
macro_rules! from_module {
    ($module:ident) => {
        SchemaEntry {
            table: $module::table,
            indexes: $module::indexes,
            foreign_keys: $module::foreign_keys,
            unique_constraints: $module::unique_constraints,
        }
    };
}

use crate::entities::{ApiKey, Organization, Team, TeamMember, User};

const SCHEMA_ENTRIES: &[SchemaEntry] = &[
    // ── Migrated to HasTableSchema (entity is the source of truth) ──
    from_entity!(Organization),
    from_entity!(User),
    from_entity!(Team),
    from_entity!(TeamMember),
    from_entity!(ApiKey),
    // ── Legacy schema modules (pending migration) ──
    from_module!(projects),
    from_module!(collections),
    from_module!(observations),
    from_module!(session_summaries),
    from_module!(file_hashes),
    from_module!(agent_sessions),
    from_module!(delegations),
    from_module!(tool_calls),
    from_module!(checkpoints),
    from_module!(error_patterns),
    from_module!(error_pattern_matches),
    from_module!(project_issues),
    from_module!(issue_comments),
    from_module!(issue_labels),
    from_module!(issue_label_assignments),
    from_module!(plans),
    from_module!(plan_versions),
    from_module!(plan_reviews),
    from_module!(repositories),
    from_module!(branches),
    from_module!(worktrees),
    from_module!(agent_worktree_assignments),
];

impl Schema {
    /// Build the canonical full schema definition.
    #[must_use]
    pub fn definition() -> Self {
        Self {
            tables: Self::tables(),
            fts: Self::fts_def(),
            indexes: Self::indexes(),
            foreign_keys: Self::foreign_keys(),
            unique_constraints: Self::unique_constraints(),
        }
    }

    fn tables() -> Vec<TableDef> {
        SCHEMA_ENTRIES.iter().map(|entry| (entry.table)()).collect()
    }

    fn fts_def() -> Option<FtsDef> {
        Some(FtsDef {
            virtual_table_name: "observations_fts".to_owned(),
            content_table: "observations".to_owned(),
            content_columns: vec!["content".to_owned()],
            id_column: "id".to_owned(),
        })
    }

    fn indexes() -> Vec<IndexDef> {
        SCHEMA_ENTRIES
            .iter()
            .flat_map(|entry| (entry.indexes)().into_iter())
            .collect()
    }

    fn foreign_keys() -> Vec<ForeignKeyDef> {
        SCHEMA_ENTRIES
            .iter()
            .flat_map(|entry| (entry.foreign_keys)().into_iter())
            .collect()
    }

    fn unique_constraints() -> Vec<UniqueConstraintDef> {
        SCHEMA_ENTRIES
            .iter()
            .flat_map(|entry| (entry.unique_constraints)().into_iter())
            .collect()
    }
}
