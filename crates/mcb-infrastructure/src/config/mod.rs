//! Configuration — concrete implementation (CA/DI).
//!
//! All access goes through `mcb_domain::utils::config` helpers →
//! `mcb_domain::registry::config::resolve_config_provider()`.
//!
//! Types are `pub` (needed for downcast at composition root).
//! Loader/validation are private implementation details.

pub mod app;
pub mod infrastructure;
mod loader;
pub mod mode;
mod provider;
pub mod system;
mod validation;
