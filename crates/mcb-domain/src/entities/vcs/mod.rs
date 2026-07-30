//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#core-entities)
//!
//! Version Control System (VCS) domain entities
//!
//! Includes repositories, branches, commits, and diffs.

mod branch;
mod commit;
mod diff;
mod vcs_repo;

pub use branch::VcsBranch;
pub use commit::{VcsCommit, VcsCommitInput};
pub use diff::{DiffStatus, FileDiff, RefDiff};
pub use vcs_repo::VcsRepository;
