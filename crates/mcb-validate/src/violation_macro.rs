//! Violation Definition Macro
//!
//! Provides a declarative macro for defining violation enums with
//! automatic trait implementations.
//!
mod extract_file_path;
mod impl_validator_macro;
/// Field-formatting traits used by violation message template expansion.
/// Helper module for formatting violation fields.
pub mod violation_field_fmt;

pub use extract_file_path::ExtractFilePath;

mod define_violations;
