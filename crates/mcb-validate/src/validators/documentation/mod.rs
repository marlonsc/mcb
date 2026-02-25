//! Documentation Completeness Validation
//!
//! Validates documentation:
//! - All pub items have rustdoc (///)
//! - Module-level documentation (//!)
//! - Example code blocks for traits

mod helpers;
mod validator;

pub use self::validator::{DocumentationValidator, DocumentationViolation};
