//! Validation constants
//!
//! Organized by semantic domain:
//!
//! - [`common`] — Cross-cutting code patterns (comments, test paths, declarations)
//! - [`architecture`] — Architecture layer path fragments
//! - [`labels`] — Pending-task and stub detection labels
//! - [`allowlists`] — Validation skip-lists and generic names
//! - [`duplication`] — Clone detection fingerprinting keywords

/// Cross-cutting code patterns shared across multiple validators.
pub mod common;

/// Architecture layer path fragments.
pub mod architecture;

/// Pending-task and stub detection labels.
pub mod labels;

/// Validation skip-lists and generic names.
pub mod allowlists;

/// Clone detection fingerprinting keywords.
pub mod duplication;

// ============================================================================
// Re-exports for backward compatibility
// ============================================================================

// Labels (used by quality/comments.rs, implementation/stubs.rs, test_quality.rs)
pub use labels::{
    PENDING_LABEL_FIXME, PENDING_LABEL_HACK, PENDING_LABEL_TODO, PENDING_LABEL_XXX,
    REPORT_TEST_PENDING_LABEL, STUB_PANIC_LABEL,
};

// Allowlists
pub use allowlists::{
    GENERIC_TYPE_NAMES, REFACTORING_SKIP_DIR_PATTERNS, REFACTORING_SKIP_FILES, UTILITY_TYPES,
};

// Duplication (used by duplication/detector.rs)
pub use duplication::DUPLICATION_KEYWORDS;

// ============================================================================
// PMAT Integration (top-level, no sub-module needed)
// ============================================================================

/// Default cyclomatic complexity threshold.
pub const DEFAULT_COMPLEXITY_THRESHOLD: u32 = 15;

/// Default TDG score threshold (0-100, higher is worse).
pub const DEFAULT_TDG_THRESHOLD: u32 = 50;

/// Default max lines per file before triggering a size violation.
pub const DEFAULT_MAX_FILE_LINES: usize = 500;
