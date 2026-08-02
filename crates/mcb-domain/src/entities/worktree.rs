//! Worktree and agent-worktree assignment entities.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!
//!
//! This module defines the entities for managing git worktree checkouts and their
//! association with agent sessions to prevent concurrent workspace conflicts.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Worktree
// ---------------------------------------------------------------------------

crate::define_entity! {
    /// A git worktree checkout associated with a repository and branch.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Worktree { id, created_at, updated_at } {
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
}

crate::define_string_enum! {
    /// Lifecycle status of a worktree.
    pub enum WorktreeStatus [strum = "snake_case", serde = "snake_case", schema] {
        /// Worktree is available for use.
        Active,
        /// Worktree is currently in use by an agent.
        InUse,
        /// Worktree has been pruned / removed.
        Pruned,
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
