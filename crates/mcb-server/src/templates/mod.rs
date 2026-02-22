//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Templating system for MCB Server (Axum-based).

mod context;
pub(crate) mod embedded;
pub(crate) mod engine;
mod fairing;
mod metadata;
mod template;

pub use engine::Engines;
pub use metadata::Metadata;
pub use template::{Template, init_axum_context};
