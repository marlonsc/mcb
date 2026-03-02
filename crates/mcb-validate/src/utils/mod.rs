//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Centralized utility modules for mcb-validate.

/// Filesystem utilities (YAML collection, directory traversal).
pub mod fs;
/// Line/interval range helpers (overlap checks).
pub mod range;
/// Source code analysis utilities (function extraction, brace tracking, block scanning).
pub mod source;
/// Validation report utilities (conversion, filtering).
pub mod validation_report;
/// YAML utilities (embedded rules, parsing).
pub mod yaml;
