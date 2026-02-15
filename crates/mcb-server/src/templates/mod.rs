//! Templating system integration for MCB Server.
//!
//! This module provides a `Rocket` fairing and supporting structures to integrate
//! various templating engines (Handlebars, etc.) into the Rocket application.
//! It handles context management, engine selection, and automatic reloading
//! in debug mode.

mod context;
mod context_macro;
pub(crate) mod embedded;
pub(crate) mod engine;
mod fairing;
mod metadata;
mod template;

pub use engine::Engines;
pub use metadata::Metadata;
pub use template::Template;
