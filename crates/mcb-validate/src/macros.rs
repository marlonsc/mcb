//!
//! **Documentation**: [docs/modules/validate.md](../../../docs/modules/validate.md)
//!
//! Violation runtime types (field formatting, file path extraction).
//!
//! The declarative macros (`define_violations!`, `impl_validator!`) live in
//! `crate::validators::macros`; this module provides the runtime helper types
//! that generated code references via `$crate::macros::*` paths.

mod extract_file_path;
/// Field-formatting traits used by violation message template expansion.
pub mod violation_field_fmt;

pub use extract_file_path::ExtractFilePath;
