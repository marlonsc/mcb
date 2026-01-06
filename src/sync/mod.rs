//! Synchronization system with cross-process coordination
//!
//! Provides lockfile-based synchronization to coordinate multiple MCP instances.

pub mod lockfile;
pub mod manager;

pub use lockfile::{CodebaseLockManager, LockMetadata};
pub use manager::{SyncConfig, SyncManager, SyncStats};
