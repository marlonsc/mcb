//!
//! Validation constants centralized from mcb-validate.
//!
//! All validation-specific constants: rule engine infrastructure, code analysis
//! patterns, detection patterns, and duplication analysis.
//!
//! Organized into submodules by concern.

/// Architecture path fragments, clean architecture naming, and linter integration.
mod architecture;
/// Severity levels, categories, and validator names.
mod categories;
/// Detection patterns: quality, SOLID, KISS, refactoring, implementation,
/// documentation, async, performance, and organization.
mod detection;
/// Clone/duplication detection constants.
mod duplication;
/// Cross-cutting code patterns (prefixes, tokens, skip lists).
mod patterns;
/// YAML rule field names, GRL keywords, engine types, and rule defaults.
mod rules;

pub use architecture::*;
pub use categories::*;
pub use detection::*;
pub use duplication::*;
pub use patterns::*;
pub use rules::*;
