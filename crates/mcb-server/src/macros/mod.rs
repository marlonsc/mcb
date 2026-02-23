//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Server macros organized by usage context.
//!
//! Sub-modules group macros by the server area they serve:
//! - [`args`]: Tool argument derive macros (used by `args/`)
//! - [`browse`]: Axum browse endpoint macros (used by `admin/browse`)
//! - [`dispatch`]: Entity handler dispatch macros (used by `handlers/entities/`)
//! - [`web`]: Template macros (used by `templates/`)

#[macro_use]
mod args;
#[macro_use]
mod browse;
#[macro_use]
mod dispatch;
#[macro_use]
mod handlers;
#[macro_use]
mod web;
