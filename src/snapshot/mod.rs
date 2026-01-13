//! Snapshot management for incremental codebase tracking
//!
//! Tracks file changes using SHA256 hashing for efficient incremental sync.
//! Avoids reprocessing unchanged files during codebase indexing.
//!
//! ## Services
//!
//! - `SnapshotManager` - Main snapshot orchestration (backward compatible)
//! - `HashCalculator` - Computes cryptographic hashes for files
//! - `SnapshotComparator` - Compares snapshots to detect changes

mod comparator;
mod hash;
mod manager;

pub use comparator::SnapshotComparator;
pub use hash::HashCalculator;
pub use manager::SnapshotManager;
// Re-export domain types
pub use crate::domain::types::{CodebaseSnapshot, FileSnapshot, SnapshotChanges};
