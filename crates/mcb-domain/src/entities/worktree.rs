//! Worktree and agent-worktree assignment entities.
//!
//! A worktree is an additional checkout of a repository branch on disk.
//! Agent sessions can be assigned to worktrees to avoid concurrent
//! modifications to the same working directory.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Worktree
// ---------------------------------------------------------------------------

/// A git worktree checkout associated with a repository and branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    /// Unique identifier (UUID).
    pub id: String,
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
    /// Timestamp when the worktree was created (Unix epoch).
    pub created_at: i64,
    /// Timestamp of last status change (Unix epoch).
    pub updated_at: i64,
}

/// Lifecycle status of a worktree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::InUse => "in_use",
            Self::Pruned => "pruned",
        }
    }
}

impl std::fmt::Display for WorktreeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for WorktreeStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "in_use" => Ok(Self::InUse),
            "pruned" => Ok(Self::Pruned),
            _ => Err(format!("Unknown worktree status: {s}")),
        }
    }
}

// ---------------------------------------------------------------------------
// AgentWorktreeAssignment
// ---------------------------------------------------------------------------

/// Tracks which agent session is/was assigned to which worktree.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
