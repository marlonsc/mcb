//! Infrastructure service ports.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)

/// Configuration provider ports.
pub mod config;
/// Event bus provider ports.
pub mod events;
/// GraphQL schema provider ports.
pub mod graphql;
/// Lifecycle management and health check ports.
pub mod lifecycle;
/// Logging ports.
pub mod logging;
/// Database migration ports.
pub mod migrations;
/// Provider routing ports.
pub mod routing;
/// Snapshot and sync provider ports.
pub mod sync;

// Re-exports for canonical access via `ports::infrastructure::{...}`
pub use config::ConfigProvider;
pub use events::{DomainEventStream, EventBusProvider};
pub use graphql::{GraphQLSchemaProvider, SharedGraphQLSchemaProvider};
pub use lifecycle::{
    DependencyHealth, DependencyHealthCheck, ExtendedHealthResponse, LifecycleManaged,
    PortServiceState, ShutdownCoordinator,
};
pub use logging::{LogLevel, OperationLogger};
pub use migrations::{MigrationProvider, SharedMigrationProvider};
pub use routing::{ProviderContext, ProviderHealthStatus, ProviderRouter};
pub use sync::{
    SharedSyncCoordinator, SnapshotProvider, SyncCoordinator, SyncOptions, SyncProvider, SyncResult,
};
