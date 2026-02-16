//! Violation runtime types (field formatting, file path extraction).
//!
//! The declarative macros (`define_violations!`, `impl_validator!`) live in
//! `crate::macros`; this module provides the runtime helper types that the
//! generated code references via `$crate::violation_macro::*` paths.
//!
#[path = "validators/macros/extract_file_path.rs"]
mod extract_file_path;
/// Field-formatting traits used by violation message template expansion.
/// Helper module for formatting violation fields.
#[path = "validators/macros/violation_field_fmt.rs"]
pub mod violation_field_fmt;

pub use extract_file_path::ExtractFilePath;
