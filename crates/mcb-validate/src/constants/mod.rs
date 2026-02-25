//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Validation constants
//!
//! Organized by semantic domain:
//!
//! - [`common`] — Cross-cutting code patterns (comments, test paths, declarations)
//! - [`architecture`] — Architecture layer path fragments
//! - [`labels`] — Pending-task and stub detection labels
//! - [`allowlists`] — Validation skip-lists and generic names
//! - [`duplication`] — Clone detection fingerprinting keywords
//!
//! - [`ca`] — Clean Architecture naming and layout

/// Cross-cutting code patterns shared across multiple validators.
pub mod common;

/// Architecture layer path fragments.
pub mod architecture;

/// Clean Architecture naming and layout constants.
pub mod ca;

/// Pending-task and stub detection labels.
pub mod labels;

/// Validation skip-lists and generic names.
pub mod allowlists;

/// Clone detection fingerprinting keywords.
pub mod duplication;

/// YAML rule field names and default values.
pub mod rules;

/// Rule engine type identifiers.
pub mod engines;

/// Severity and violation category string constants.
pub mod severities;

/// Validator category name constants.
pub mod validators;

/// Linter integration (Clippy/Cargo) constants.
pub mod linters;

// ============================================================================
// PMAT Integration (top-level, no sub-module needed)
// ============================================================================

/// Default values for validation settings.
pub mod defaults;
