//! # MCP Context Browser - Utilities Layer
//!
//! **Layer 0** — Innermost workspace crate with zero domain knowledge.
//!
//! Shared utilities, constants, and helper functions used across all MCB crates.
//! This crate has NO dependencies on any other `mcb-*` workspace crate.
//!
//! ## Architecture
//!
//! | Component | Description |
//! | ----------- | ------------- |
//! | [`error`] | Utility error types (`UtilsError`) |
//! | [`utils`] | Shared helper functions |
//! | [`constants`] | Workspace-wide constants |
//!
//! ## Design Principles
//!
//! - **Zero domain knowledge** — never imports from `mcb-domain` or any other workspace crate
//! - **Pure utilities** — no business logic, no entity definitions
//! - **Self-contained error types** — `UtilsError` with `thiserror`, converted via `From` in consumers

/// Crate-internal macros (must precede consumers).
#[macro_use]
mod macros;

/// Workspace-wide constants.
pub mod constants;
/// Utility error types.
pub mod error;
/// Shared helper functions.
pub mod utils;

// Re-export core error types for convenience.
pub use error::{Result, UtilsError};
