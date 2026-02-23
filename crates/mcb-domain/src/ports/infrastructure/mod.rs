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
//! | [`AuthServiceInterface`] | Authentication and token services |
//! | [`EventBusProvider`] | Event publish/subscribe services |
//! | [`SystemMetricsCollectorInterface`] | System metrics collection |
//! | [`StateStoreProvider`] | Key-value state persistence |
//! | [`ProviderRouter`] | Provider routing and selection services |
//!

/// Authentication service port
pub mod auth;
/// Event bus provider port
pub mod events;
pub mod lifecycle;
/// Operation logging port (level + context + message + optional detail).
pub mod logging;
/// System metrics collector port
pub mod metrics;
/// Provider routing and selection port
pub mod routing;
/// Snapshot management infrastructure port
pub mod snapshot;
/// Key-value state store port
mod state_store;
/// File synchronization infrastructure port
pub mod sync;

// Re-export infrastructure ports
pub use auth::AuthServiceInterface;
pub use events::{DomainEventStream, EventBusProvider};
pub use lifecycle::{
    DependencyHealth, DependencyHealthCheck, ExtendedHealthResponse, LifecycleManaged,
    PortServiceState, ShutdownCoordinator,
};
pub use logging::{LogLevel, OperationLogger};
pub use metrics::{SystemMetrics, SystemMetricsCollectorInterface};
pub use routing::{ProviderContext, ProviderHealthStatus, ProviderRouter};
pub use snapshot::{SnapshotProvider, SyncProvider};
pub use state_store::StateStoreProvider;
pub use sync::{SharedSyncCoordinator, SyncCoordinator, SyncOptions, SyncResult};
