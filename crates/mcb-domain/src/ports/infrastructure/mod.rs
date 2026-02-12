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
//! | [`SyncCoordinator`] | File system synchronization services |
//! | [`SnapshotProvider`] | Codebase snapshot management |
//! | [`AuthServiceInterface`] | Authentication and token services |
//! | [`EventBusProvider`] | Event publish/subscribe services |
//! | [`SystemMetricsCollectorInterface`] | System metrics collection |
//! | [`PerformanceMetricsCollector`](crate::ports::infrastructure::performance::PerformanceMetricsCollector) | Provider performance metrics (Prometheus) |
//! | [`StateStoreProvider`] | Key-value state persistence |
//! | [`ProviderRouter`] | Provider routing and selection services |
//! | [`DatabaseExecutor`] | SQL execution (repositories use via DI, no direct driver) |

/// Authentication service port
pub mod auth;
/// Database executor port (SQL execution abstraction)
pub mod database;
/// Event bus provider port
pub mod events;
/// System metrics collector port
pub mod metrics;
/// Performance metrics collector port (Prometheus histograms/counters)
pub mod performance;
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
pub use database::{DatabaseExecutor, DatabaseProvider, SqlParam, SqlRow};
pub use events::{DomainEventStream, EventBusProvider};
pub use metrics::{SystemMetrics, SystemMetricsCollectorInterface};
pub use performance::PerformanceMetricsCollector;
pub use routing::{ProviderContext, ProviderHealthStatus, ProviderRouter};
pub use snapshot::{SnapshotProvider, SyncProvider};
pub use state_store::StateStoreProvider;
pub use sync::{SharedSyncCoordinator, SyncCoordinator, SyncOptions, SyncResult};
