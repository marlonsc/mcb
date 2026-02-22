//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Severity and category string constants.
//!
//! Used by rule engines, linter integration, and violation classification
//! for mapping string representations to typed enums.

// ============================================================================
// Severity Level Strings
// ============================================================================

/// Severity string: error.
pub const SEVERITY_ERROR: &str = "error";

/// Severity string: warning.
pub const SEVERITY_WARNING: &str = "warning";

/// Severity string: info/informational.
pub const SEVERITY_INFO: &str = "info";

// ============================================================================
// Violation Category Strings
// ============================================================================

/// Category: architecture violations.
pub const CATEGORY_ARCHITECTURE: &str = "architecture";

/// Category: clean architecture violations.
pub const CATEGORY_CLEAN_ARCHITECTURE: &str = "clean-architecture";

/// Category: code organization.
pub const CATEGORY_ORGANIZATION: &str = "organization";

/// Category: SOLID principles.
pub const CATEGORY_SOLID: &str = "solid";

/// Category: dependency injection.
pub const CATEGORY_DI: &str = "di";

/// Category: configuration quality.
pub const CATEGORY_CONFIGURATION: &str = "configuration";

/// Category: web framework patterns.
pub const CATEGORY_WEB_FRAMEWORK: &str = "web-framework";

/// Category: performance issues.
pub const CATEGORY_PERFORMANCE: &str = "performance";

/// Category: async patterns.
pub const CATEGORY_ASYNC: &str = "async";

/// Category: documentation completeness.
pub const CATEGORY_DOCUMENTATION: &str = "documentation";

/// Category: testing quality.
pub const CATEGORY_TESTING: &str = "testing";

/// Category: naming conventions.
pub const CATEGORY_NAMING: &str = "naming";

/// Category: KISS principle.
pub const CATEGORY_KISS: &str = "kiss";

/// Category: refactoring opportunities.
pub const CATEGORY_REFACTORING: &str = "refactoring";

/// Category: migration issues.
pub const CATEGORY_MIGRATION: &str = "migration";

/// Category: error boundary patterns.
pub const CATEGORY_ERROR_BOUNDARY: &str = "error_boundary";

/// Category: implementation patterns.
pub const CATEGORY_IMPLEMENTATION: &str = "implementation";

/// Category: PMAT (process maturity).
pub const CATEGORY_PMAT: &str = "pmat";
