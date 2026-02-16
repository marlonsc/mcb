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
//! because mcb-server needs them for `AdminState`. These implement traits from
//! mcb-application but are infrastructure concerns, not external providers.

// Admin module - partially exported for mcb-server
pub mod admin;

/// Infrastructure factory helpers.
/// Infrastructure factory
pub mod factory;

// Internal modules - implementations NOT exported
// pub(crate) mod events;
pub(crate) mod lifecycle;

// Public data types (NOT implementations) - these are pure DTOs
// Admin types - exported for mcb-server AdminState
pub use admin::{AtomicPerformanceMetrics, DefaultIndexingOperations};
pub use factory::default_event_bus;
pub use lifecycle::{ServiceInfo, ServiceManager, ServiceManagerError};
