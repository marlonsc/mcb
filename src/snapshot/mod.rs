//! Snapshot management for incremental codebase tracking
//!
//! Tracks file changes using SHA256 hashing for efficient incremental sync.
//! Avoids reprocessing unchanged files during codebase indexing.

mod manager;

pub use manager::SnapshotManager;
// Re-export domain types
pub use crate::domain::types::{CodebaseSnapshot, FileSnapshot, SnapshotChanges};
