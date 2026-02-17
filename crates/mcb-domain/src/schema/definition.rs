use super::types::{ForeignKeyDef, FtsDef, IndexDef, Schema, TableDef, UniqueConstraintDef};
use super::{
    agent_sessions, agent_worktree_assignments, api_keys, branches, checkpoints, collections,
    delegations, error_pattern_matches, error_patterns, file_hashes, issue_comments,
    issue_label_assignments, issue_labels, observations, organizations, plan_reviews,
    plan_versions, plans, project_issues, projects, repositories, session_summaries, team_members,
    teams, tool_calls, users, worktrees,
};

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
        vec![
            organizations::table(),
            users::table(),
            teams::table(),
            team_members::table(),
            api_keys::table(),
            projects::table(),
            collections::table(),
            observations::table(),
            session_summaries::table(),
            file_hashes::table(),
            agent_sessions::table(),
            delegations::table(),
            tool_calls::table(),
            checkpoints::table(),
            error_patterns::table(),
            error_pattern_matches::table(),
            project_issues::table(),
            issue_comments::table(),
            issue_labels::table(),
            issue_label_assignments::table(),
            plans::table(),
            plan_versions::table(),
            plan_reviews::table(),
            repositories::table(),
            branches::table(),
            worktrees::table(),
            agent_worktree_assignments::table(),
        ]
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
        let mut indexes = Vec::new();
        indexes.extend(organizations::indexes());
        indexes.extend(users::indexes());
        indexes.extend(teams::indexes());
        indexes.extend(team_members::indexes());
        indexes.extend(api_keys::indexes());
        indexes.extend(projects::indexes());
        indexes.extend(collections::indexes());
        indexes.extend(observations::indexes());
        indexes.extend(session_summaries::indexes());
        indexes.extend(file_hashes::indexes());
        indexes.extend(agent_sessions::indexes());
        indexes.extend(delegations::indexes());
        indexes.extend(tool_calls::indexes());
        indexes.extend(checkpoints::indexes());
        indexes.extend(error_patterns::indexes());
        indexes.extend(error_pattern_matches::indexes());
        indexes.extend(project_issues::indexes());
        indexes.extend(issue_comments::indexes());
        indexes.extend(issue_labels::indexes());
        indexes.extend(issue_label_assignments::indexes());
        indexes.extend(plans::indexes());
        indexes.extend(plan_versions::indexes());
        indexes.extend(plan_reviews::indexes());
        indexes.extend(repositories::indexes());
        indexes.extend(branches::indexes());
        indexes.extend(worktrees::indexes());
        indexes.extend(agent_worktree_assignments::indexes());
        indexes
    }

    fn foreign_keys() -> Vec<ForeignKeyDef> {
        let mut fks = Vec::new();
        fks.extend(organizations::foreign_keys());
        fks.extend(users::foreign_keys());
        fks.extend(teams::foreign_keys());
        fks.extend(team_members::foreign_keys());
        fks.extend(api_keys::foreign_keys());
        fks.extend(projects::foreign_keys());
        fks.extend(collections::foreign_keys());
        fks.extend(observations::foreign_keys());
        fks.extend(session_summaries::foreign_keys());
        fks.extend(file_hashes::foreign_keys());
        fks.extend(agent_sessions::foreign_keys());
        fks.extend(delegations::foreign_keys());
        fks.extend(tool_calls::foreign_keys());
        fks.extend(checkpoints::foreign_keys());
        fks.extend(error_patterns::foreign_keys());
        fks.extend(error_pattern_matches::foreign_keys());
        fks.extend(project_issues::foreign_keys());
        fks.extend(issue_comments::foreign_keys());
        fks.extend(issue_labels::foreign_keys());
        fks.extend(issue_label_assignments::foreign_keys());
        fks.extend(plans::foreign_keys());
        fks.extend(plan_versions::foreign_keys());
        fks.extend(plan_reviews::foreign_keys());
        fks.extend(repositories::foreign_keys());
        fks.extend(branches::foreign_keys());
        fks.extend(worktrees::foreign_keys());
        fks.extend(agent_worktree_assignments::foreign_keys());
        fks
    }

    fn unique_constraints() -> Vec<UniqueConstraintDef> {
        let mut uniques = Vec::new();
        uniques.extend(organizations::unique_constraints());
        uniques.extend(users::unique_constraints());
        uniques.extend(teams::unique_constraints());
        uniques.extend(team_members::unique_constraints());
        uniques.extend(api_keys::unique_constraints());
        uniques.extend(projects::unique_constraints());
        uniques.extend(collections::unique_constraints());
        uniques.extend(observations::unique_constraints());
        uniques.extend(session_summaries::unique_constraints());
        uniques.extend(file_hashes::unique_constraints());
        uniques.extend(agent_sessions::unique_constraints());
        uniques.extend(delegations::unique_constraints());
        uniques.extend(tool_calls::unique_constraints());
        uniques.extend(checkpoints::unique_constraints());
        uniques.extend(error_patterns::unique_constraints());
        uniques.extend(error_pattern_matches::unique_constraints());
        uniques.extend(project_issues::unique_constraints());
        uniques.extend(issue_comments::unique_constraints());
        uniques.extend(issue_labels::unique_constraints());
        uniques.extend(issue_label_assignments::unique_constraints());
        uniques.extend(plans::unique_constraints());
        uniques.extend(plan_versions::unique_constraints());
        uniques.extend(plan_reviews::unique_constraints());
        uniques.extend(repositories::unique_constraints());
        uniques.extend(branches::unique_constraints());
        uniques.extend(worktrees::unique_constraints());
        uniques.extend(agent_worktree_assignments::unique_constraints());
        uniques
    }
}
