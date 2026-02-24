//!
//! Infrastructure Services
//!
//! Infrastructure service implementations for port traits defined in mcb-domain.
//! Concrete types are composed in the DI bootstrap module or `loco_app.rs`.
pub mod admin;
pub mod factory;
pub use admin::DefaultIndexingOperations;
pub use factory::default_event_bus;
