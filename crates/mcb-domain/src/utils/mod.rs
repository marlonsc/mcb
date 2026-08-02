//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Domain-specific utility modules.
//!
//! Pure utilities (fs, id, naming, path, sensitivity, time, vcs_context)
//! live in `mcb-utils` (Layer 0). Import them via `mcb_utils::utils::*`.

/// Complexity and analysis utilities.
pub mod analysis;
/// Configuration helpers — simplified CA/DI access.
pub mod config;
/// Project type detection helpers.
pub mod project_type;
/// Submodule path helpers.
pub mod submodule;
/// MCP text extraction utilities (extract_text, extract_text_with_sep).
pub mod text;

#[cfg(any(test, feature = "test-utils"))]
/// Test infrastructure — fixtures, constants, service-config helpers.
pub mod tests;
