//! Violation runtime types (field formatting, file path extraction).
//!
//! The declarative macros (`define_violations!`, `impl_validator!`) live in
//! `crate::macros`; this module provides the runtime helper types that the
//! generated code references via `$crate::violation_macro::*` paths.
//!
mod extract_file_path;
/// Field-formatting traits used by violation message template expansion.
/// Helper module for formatting violation fields.
pub mod violation_field_fmt;

pub use extract_file_path::ExtractFilePath;
