//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md)
//!
//! Infrastructure Ports
//!
//! Ports for infrastructure services that provide technical capabilities
//! to the domain. These interfaces define contracts for file synchronization,
//! snapshot management, and other cross-cutting infrastructure concerns.
//!
//! ## Infrastructure Ports
//!
//! | Port | Description |
//! | ------ | ------------- |
//! | [`SyncCoordinator`] | File system synchronization services |
//! | [`SnapshotProvider`] | Codebase snapshot management |
//! | [`ProviderRouter`] | Provider routing and selection services |
//!

/// Event bus provider port
pub mod events;
pub mod lifecycle;
/// Operation logging port (level + context + message + optional detail).
pub mod logging;
/// Provider routing and selection port
pub mod routing;
/// Snapshot management infrastructure port
pub mod snapshot;
/// File synchronization infrastructure port
pub mod sync;

// Re-export infrastructure ports
pub use events::{DomainEventStream, EventBusProvider};
pub use lifecycle::{
    DependencyHealth, DependencyHealthCheck, ExtendedHealthResponse, LifecycleManaged,
    PortServiceState, ShutdownCoordinator,
};
pub use logging::{LogLevel, OperationLogger};
pub use routing::{ProviderContext, ProviderHealthStatus, ProviderRouter};
pub use snapshot::{SnapshotProvider, SyncProvider};
pub use sync::{SharedSyncCoordinator, SyncCoordinator, SyncOptions, SyncResult};
