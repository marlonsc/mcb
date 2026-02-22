//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#core-entities)
//!
use std::path::PathBuf;

use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Represents the status of a file in a version control diff.
///
/// Indicates whether a file was added, modified, deleted, or renamed
/// in a particular commit or change set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum DiffStatus {
    /// File was newly added in this diff
    #[display("added")]
    Added,
    /// File was modified in this diff
    #[display("modified")]
    Modified,
    /// File was deleted in this diff
    #[display("deleted")]
    Deleted,
    /// File was renamed in this diff
    #[display("renamed")]
    Renamed,
}

/// Represents a single file's changes in a version control diff.
///
/// Contains metadata about a file that was modified, added, deleted, or renamed,
/// including its path, status, and the number of lines added and deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    /// Unique identifier for this file diff
    pub id: String,
    /// Path to the file in the repository
    pub path: PathBuf,
    /// Status of the file (added, modified, deleted, or renamed)
    pub status: DiffStatus,
    /// Number of lines added in this file
    pub additions: usize,
    /// Number of lines deleted in this file
    pub deletions: usize,
}

/// Represents the complete diff between two git references.
///
/// Contains all file changes between a base reference (e.g., main branch)
/// and a head reference (e.g., feature branch), along with aggregate statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefDiff {
    /// Unique identifier for this reference diff
    pub id: String,
    /// The base reference (e.g., branch name or commit hash)
    pub base_ref: String,
    /// The head reference (e.g., branch name or commit hash)
    pub head_ref: String,
    /// List of all file changes in this diff
    pub files: Vec<FileDiff>,
    /// Total number of lines added across all files
    pub total_additions: usize,
    /// Total number of lines deleted across all files
    pub total_deletions: usize,
}
