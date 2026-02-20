//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Centralized utility modules for mcb-validate.

/// Filesystem utilities (YAML collection, directory traversal).
pub mod fs;
/// Naming convention checks (`CamelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`).
pub mod naming;
/// Source code analysis utilities (function extraction, brace tracking, block scanning).
pub mod source;
/// Validation report utilities (conversion, filtering).
pub mod validation_report;
/// YAML utilities (embedded rules, parsing).
pub mod yaml;
