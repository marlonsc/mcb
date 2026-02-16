//! Documentation Completeness Validation
//!
//! Validates documentation:
//! - All pub items have rustdoc (///)
//! - Module-level documentation (//!)
//! - Example code blocks for traits

pub mod constants;
mod validator;

pub use self::validator::{DocumentationValidator, DocumentationViolation};
