//! Validation Port Interfaces
//!
//! Validation types, language identification, configuration, and validator traits.

/// Validation configuration and check runner.
mod config;
/// Supported programming languages for scanners and validators.
mod language_id;
/// Common validation types and traits.
mod types;
/// Validator trait for implementing codebase validation rules.
mod validator;

pub use config::*;
pub use language_id::*;
pub use types::*;
pub use validator::*;
