//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!

// Legacy schema modules — pending migration to impl_table_schema! on entities.
// Migrated: organizations, users, teams, team_members, api_keys (deleted).
mod agent_sessions;
mod agent_worktree_assignments;
mod branches;
mod checkpoints;
mod collections;
mod definition;
mod delegations;
mod error_pattern_matches;
mod error_patterns;
mod file_hashes;
mod issue_comments;
mod issue_label_assignments;
mod issue_labels;
mod observations;
mod plan_reviews;
mod plan_versions;
mod plans;
mod project_issues;
mod projects;
mod repositories;
mod session_summaries;
mod tool_calls;
/// Canonical schema model types and DDL generation traits.
pub mod types;
mod worktrees;

pub use types::*;
