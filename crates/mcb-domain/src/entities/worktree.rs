//! Worktree and agent-worktree assignment entities.
//!
//! This module defines the entities for managing git worktree checkouts and their
//! association with agent sessions to prevent concurrent workspace conflicts.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Worktree
// ---------------------------------------------------------------------------

/// A git worktree checkout associated with a repository and branch.
///
use super::EntityMetadata;

/// A git worktree checkout associated with a repository and branch.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Worktree {
    /// Common entity metadata (id, timestamps).
    #[serde(flatten)]
    pub metadata: EntityMetadata,
    /// Repository this worktree belongs to.
    pub repository_id: String,
    /// Branch checked out in this worktree.
    pub branch_id: String,
    /// Filesystem path of the worktree.
    pub path: String,
    /// Current status of the worktree.
    pub status: WorktreeStatus,
    /// Agent session currently assigned to this worktree (if any).
    pub assigned_agent_id: Option<String>,
}

impl_base_entity!(Worktree);

/// Lifecycle status of a worktree.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    strum_macros::Display,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
pub enum WorktreeStatus {
    /// Worktree is available for use.
    Active,
    /// Worktree is currently in use by an agent.
    InUse,
    /// Worktree has been pruned / removed.
    Pruned,
}

impl WorktreeStatus {
    /// Returns the string representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

// ---------------------------------------------------------------------------
// AgentWorktreeAssignment
// ---------------------------------------------------------------------------

/// Tracks which agent session is/was assigned to which worktree.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentWorktreeAssignment {
    /// Unique identifier (UUID).
    pub id: String,
    /// Agent session that was assigned.
    pub agent_session_id: String,
    /// Worktree that was assigned.
    pub worktree_id: String,
    /// When the assignment started (Unix epoch).
    pub assigned_at: i64,
    /// When the assignment ended (Unix epoch). `None` if still active.
    pub released_at: Option<i64>,
}
