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
//! | [`DatabaseExecutor`] | SQL execution (repositories use via DI, no direct driver) |

pub mod admin;
/// Authentication service port
pub mod auth;
/// Database executor port (SQL execution abstraction)
pub mod database;
/// Event bus provider port
pub mod events;
pub mod lifecycle;
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
pub use admin::{
    CacheAdminInterface, EmbeddingAdminInterface, LanguageAdminInterface, ProviderInfo,
    VectorStoreAdminInterface,
};
pub use auth::AuthServiceInterface;
pub use database::{DatabaseExecutor, DatabaseProvider, SqlParam, SqlRow};
pub use events::{DomainEventStream, EventBusProvider};
pub use lifecycle::{
    DependencyHealth, DependencyHealthCheck, ExtendedHealthResponse, LifecycleManaged,
    PortServiceState, ShutdownCoordinator,
};
pub use metrics::{SystemMetrics, SystemMetricsCollectorInterface};
pub use routing::{ProviderContext, ProviderHealthStatus, ProviderRouter};
pub use snapshot::{SnapshotProvider, SyncProvider};
pub use state_store::StateStoreProvider;
pub use sync::{SharedSyncCoordinator, SyncCoordinator, SyncOptions, SyncResult};
