//! Version Control System (VCS) domain entities
//!
//! Includes repositories, branches, commits, and diffs.

mod branch;
mod commit;
mod diff;
mod vcs_repo;

pub use branch::VcsBranch;
pub use commit::VcsCommit;
pub use diff::{DiffStatus, FileDiff, RefDiff};
pub use vcs_repo::{RepositoryId, VcsRepository};
