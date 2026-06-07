//! Cross-crate utility modules for the MCB workspace.
//!
//! **Policy**: See [`UTILITIES_POLICY.md`](./UTILITIES_POLICY.md) for naming, strictness,
//! and no-wrapper rules before adding code here.

/// Naming convention checks (`CamelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`).
pub mod naming;
/// Redaction of sensitive values in Debug/Display.
pub mod sensitivity;
/// VCS context data types for memory observations.
pub mod vcs_context;

/// Filesystem helpers (recursive file search, extension matching).
pub mod fs;
/// Deterministic ID generation (UUID v4/v5, SHA-256 content hashing).
pub mod id;
/// Canonical path utilities (workspace-relative, UTF-8 strict).
pub mod path;
/// Canonical time utilities (epoch seconds/nanos, strict).
pub mod time;

/// Range and interval utilities (line overlap checks).
pub mod range;
/// Retry utilities with exponential backoff.
pub mod retry;

/// Cryptographic hashing and token utilities.
pub mod crypto;
/// Regular expression compilation helpers.
pub mod regex;
