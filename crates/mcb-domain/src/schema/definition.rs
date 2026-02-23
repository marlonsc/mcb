//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
use super::types::{ForeignKeyDef, FtsDef, IndexDef, Schema, TableDef, UniqueConstraintDef};
use super::{
    agent_sessions, agent_worktree_assignments, api_keys, branches, checkpoints, collections,
    delegations, error_pattern_matches, error_patterns, file_hashes, index_operations,
    issue_comments, issue_label_assignments, issue_labels, observations, organizations,
    plan_reviews, plan_versions, plans, project_issues, projects, repositories, session_summaries,
    team_members, teams, tool_calls, users, worktrees,
};

struct SchemaModule {
    table: fn() -> TableDef,
    indexes: fn() -> Vec<IndexDef>,
    foreign_keys: fn() -> Vec<ForeignKeyDef>,
    unique_constraints: fn() -> Vec<UniqueConstraintDef>,
}

macro_rules! schema_modules {
    ($($module:ident),+ $(,)?) => {
        &[
            $(
                SchemaModule {
                    table: $module::table,
                    indexes: $module::indexes,
                    foreign_keys: $module::foreign_keys,
                    unique_constraints: $module::unique_constraints,
                },
            )+
        ]
    };
}

const SCHEMA_MODULES: &[SchemaModule] = schema_modules![
    organizations,
    users,
    teams,
    team_members,
    api_keys,
    projects,
    collections,
    observations,
    session_summaries,
    file_hashes,
    agent_sessions,
    delegations,
    tool_calls,
    checkpoints,
    error_patterns,
    error_pattern_matches,
    project_issues,
    issue_comments,
    issue_labels,
    issue_label_assignments,
    plans,
    plan_versions,
    plan_reviews,
    repositories,
    branches,
    worktrees,
    agent_worktree_assignments,
    index_operations,
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
        SCHEMA_MODULES
            .iter()
            .map(|module| (module.table)())
            .collect()
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
        SCHEMA_MODULES
            .iter()
            .flat_map(|module| (module.indexes)().into_iter())
            .collect()
    }

    fn foreign_keys() -> Vec<ForeignKeyDef> {
        SCHEMA_MODULES
            .iter()
            .flat_map(|module| (module.foreign_keys)().into_iter())
            .collect()
    }

    fn unique_constraints() -> Vec<UniqueConstraintDef> {
        SCHEMA_MODULES
            .iter()
            .flat_map(|module| (module.unique_constraints)().into_iter())
            .collect()
    }
}
