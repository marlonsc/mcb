//! Infrastructure Ports
//!
//! Ports for infrastructure services that provide technical capabilities
//! to the domain. These interfaces define contracts for file synchronization,
//! snapshot management, and other cross-cutting infrastructure concerns.
//!
//! ## Infrastructure Ports
//!
//! | Port | Description |
//! |------|-------------|
//! | [`SyncProvider`] | File system synchronization services |
//! | [`SnapshotProvider`] | Codebase snapshot management |

/// Snapshot management infrastructure port
pub mod snapshot;
/// File synchronization infrastructure port
pub mod sync;

// Re-export infrastructure ports
pub use snapshot::SnapshotProvider;
pub use snapshot::SyncProvider;
