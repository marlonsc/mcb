//! Infrastructure Services
//!
//! Infrastructure service implementations for production and testing.
//! The actual port traits are defined in mcb-application/ports/infrastructure.
//!
//! ## Production Defaults
//!
//! | Service | Default | Alternative |
//! |---------|---------|-------------|
//! | EventBus | `TokioBroadcastEventBus` | `NullEventBus` (testing) |
//!
//! ## Testing Defaults
//!
//! All `Null*` implementations are no-op stubs for unit testing.

pub mod admin;
pub mod auth;
pub mod events;
pub mod metrics;
pub mod snapshot;
pub mod sync;

// Re-export production implementations
pub use events::TokioBroadcastEventBus;

// Re-export Null implementations (testing)
pub use admin::{NullIndexingOperations, NullPerformanceMetrics};
pub use auth::NullAuthService;
pub use events::NullEventBus;
pub use metrics::NullSystemMetricsCollector;
pub use snapshot::NullSnapshotProvider;
pub use sync::NullSyncProvider;
