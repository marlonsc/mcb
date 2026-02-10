//! Infrastructure Services
//!
//! Infrastructure service implementations for port traits defined in mcb-application.
//!
//! ## ARCHITECTURE RULE
//!
//! **CONCRETE TYPES ARE INTERNAL ONLY.**
//!
//! All implementations are composed in the DI bootstrap module.
//! External code SHOULD use `init_app()` to get an `AppContext` with resolved services.
//! NEVER import concrete types directly from here - use the trait abstractions.
//!
//! ## Exception: Admin Types
//!
//! `AtomicPerformanceMetrics` and `DefaultIndexingOperations` are exported
//! because mcb-server needs them for AdminState. These implement traits from
//! mcb-application but are infrastructure concerns, not external providers.

// Internal modules - implementations NOT exported
// pub(crate) mod events;
pub(crate) mod lifecycle;
pub(crate) mod prometheus_metrics;

// Admin module - partially exported for mcb-server
pub mod admin;

// Public data types (NOT implementations) - these are pure DTOs
// Admin types - exported for mcb-server AdminState
pub use admin::{AtomicPerformanceMetrics, DefaultIndexingOperations};
pub use lifecycle::{ServiceInfo, ServiceManager, ServiceManagerError};
// Prometheus metrics - exported for /metrics endpoint
pub use prometheus_metrics::{PrometheusPerformanceMetrics, export_metrics};
// Event bus - exported for DI bootstrap and testing
// pub use events::TokioBroadcastEventBus;
