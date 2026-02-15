//! Cross-crate utility modules for the MCB workspace.
//!
//! **Policy**: See [`UTILITIES_POLICY.md`](./UTILITIES_POLICY.md) for naming, strictness,
//! and no-wrapper rules before adding code here.

/// Complexity and analysis utilities.
pub mod analysis;
/// ID generation, deterministic correlation (UUID v5), content hashing, and masking.
pub mod id;
/// Canonical path utilities — strict, no fallbacks.
pub mod path;
/// Project type detection helpers.
pub mod project_type;
/// Submodule path helpers.
pub mod submodule;
/// Canonical time utilities — strict, no fallbacks.
pub mod time;
/// VCS context data types for memory observations.
pub mod vcs_context;

pub use id::*;
