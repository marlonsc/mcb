//!
//! Infrastructure Services
//!
//! Infrastructure service implementations for port traits defined in mcb-domain.
//! Concrete types are composed in the DI bootstrap module or `loco_app.rs`.

// Admin module - DefaultIndexingOperations used by loco_app.rs
pub mod admin;
pub mod factory;
// Internal - DefaultShutdownCoordinator used by bootstrap.rs (tests only)
pub(crate) mod lifecycle;
pub use admin::{AtomicPerformanceMetrics, DefaultIndexingOperations};
pub use factory::default_event_bus;
