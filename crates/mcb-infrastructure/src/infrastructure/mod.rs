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
//!
//! ## Factory Functions
//!
//! Config-driven factories for contexts that don't need full `AppContext`:
//!
//! - `default_event_bus()` — in-process event bus (from config)
//! - `create_test_browse_vector_store()` — in-memory vector store for browse tests
//! - `create_test_vector_store_for_e2e()` — vector store with both provider + browser interfaces
//!
//! Use these instead of importing concrete types from `mcb-providers`.

// Admin module - partially exported for mcb-server
pub mod admin;

/// Infrastructure factory helpers.
pub mod factory;

// Internal modules - implementations NOT exported
// pub(crate) mod events;
pub(crate) mod lifecycle;

// Public data types (NOT implementations) - these are pure DTOs
// Admin types - exported for mcb-server AdminState
pub use admin::{AtomicPerformanceMetrics, DefaultIndexingOperations};
pub use factory::{
    create_test_browse_vector_store, create_test_vector_store_for_e2e, default_event_bus,
};
pub use lifecycle::{ServiceInfo, ServiceManager, ServiceManagerError};
