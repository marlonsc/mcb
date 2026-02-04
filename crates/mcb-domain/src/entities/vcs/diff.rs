use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl std::fmt::Display for DiffStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "added"),
            Self::Modified => write!(f, "modified"),
            Self::Deleted => write!(f, "deleted"),
            Self::Renamed => write!(f, "renamed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub id: String,
    pub path: PathBuf,
    pub status: DiffStatus,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefDiff {
    pub id: String,
    pub base_ref: String,
    pub head_ref: String,
    pub files: Vec<FileDiff>,
    pub total_additions: usize,
    pub total_deletions: usize,
}
