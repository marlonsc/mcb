//! Centralized utility modules for mcb-validate.

/// Filesystem utilities (YAML collection, directory traversal).
pub mod fs;
/// Naming convention checks (`CamelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`).
pub mod naming;
/// Source code analysis utilities (function extraction, brace tracking, block scanning).
pub mod source;
