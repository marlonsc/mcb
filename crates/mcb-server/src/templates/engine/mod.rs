//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
//! Template engine abstraction and management.
//!
//! This module defines the [`Engine`] trait for templating engines and the
//! [`Engines`] struct which aggregates and manages enabled engine instances.

mod handlebars_engine;
pub mod manager;

pub(crate) use manager::Engine;
pub use manager::Engines;
