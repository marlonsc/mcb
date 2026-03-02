//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Cross-crate utility modules for the MCB workspace.
//!
//! **Policy**: See [`UTILITIES_POLICY.md`](./UTILITIES_POLICY.md) for naming, strictness,
//! and no-wrapper rules before adding code here.

/// Complexity and analysis utilities.
pub mod analysis;
/// Filesystem utilities.
pub mod fs;
/// ID generation, deterministic correlation (UUID v5), content hashing, and masking.
pub mod id;
/// Naming convention checks (`CamelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`).
pub mod naming;
/// Canonical path utilities — strict, no fallbacks.
pub mod path;
/// Project type detection helpers.
pub mod project_type;
/// Redaction of sensitive values in Debug/Display.
pub mod sensitivity;
/// Submodule path helpers.
pub mod submodule;
/// MCP text extraction utilities (extract_text, extract_text_with_sep).
pub mod text;
/// Canonical time utilities — strict, no fallbacks.
pub mod time;
/// VCS context data types for memory observations.
pub mod vcs_context;

#[cfg(any(test, feature = "test-utils"))]
/// Test infrastructure — fixtures, constants, service-config helpers.
pub mod tests;
