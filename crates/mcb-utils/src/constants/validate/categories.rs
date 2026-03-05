//! Severity levels, categories, and validator names.

use super::super::define_str_consts;
use super::super::values::{
    TAG_ARCHITECTURE, TAG_ASYNC, TAG_DOCUMENTATION, TAG_NAMING, TAG_ORGANIZATION, TAG_PERFORMANCE,
    TAG_QUALITY, TAG_SOLID,
};
use super::patterns::VAL_ERROR;
use super::patterns::VAL_INFO;

// ============================================================================
// Severity Levels
// ============================================================================

/// Severity string: error.
pub const SEVERITY_ERROR: &str = VAL_ERROR;

/// Severity string: warning.
pub const SEVERITY_WARNING: &str = "warning";

/// Severity string: info/informational.
pub const SEVERITY_INFO: &str = VAL_INFO;

// ============================================================================
// Rule Categories (tag aliases + unique categories)
// ============================================================================

/// Category: architecture rules.
pub const CATEGORY_ARCHITECTURE: &str = TAG_ARCHITECTURE;
/// Category: organization rules.
pub const CATEGORY_ORGANIZATION: &str = TAG_ORGANIZATION;
/// Category: SOLID principles.
pub const CATEGORY_SOLID: &str = TAG_SOLID;
/// Category: performance rules.
pub const CATEGORY_PERFORMANCE: &str = TAG_PERFORMANCE;
/// Category: async patterns.
pub const CATEGORY_ASYNC: &str = TAG_ASYNC;
/// Category: documentation rules.
pub const CATEGORY_DOCUMENTATION: &str = TAG_DOCUMENTATION;
/// Category: naming conventions.
pub const CATEGORY_NAMING: &str = TAG_NAMING;
/// Category: code quality.
pub const CATEGORY_QUALITY: &str = TAG_QUALITY;

define_str_consts! {
    /// Category: clean architecture violations.
    CATEGORY_CLEAN_ARCHITECTURE = "clean-architecture";
    /// Category: dependency injection.
    CATEGORY_DI = "di";
    /// Category: dependency injection (long name).
    CATEGORY_DEPENDENCY_INJECTION = "dependency_injection";
    /// Category: configuration quality.
    CATEGORY_CONFIGURATION = "configuration";
    /// Category: web framework patterns.
    CATEGORY_WEB_FRAMEWORK = "web-framework";
    /// Category: web framework patterns (underscore version).
    CATEGORY_WEB_FRAMEWORK_UNDERSCORE = "web_framework";
    /// Category: testing quality.
    CATEGORY_TESTING = "testing";
    /// Category: metrics and statistics.
    CATEGORY_METRICS = "metrics";
    /// Category: KISS principle.
    CATEGORY_KISS = "kiss";
    /// Category: refactoring opportunities.
    CATEGORY_REFACTORING = "refactoring";
    /// Category: migration issues.
    CATEGORY_MIGRATION = "migration";
    /// Category: error boundary patterns.
    CATEGORY_ERROR_BOUNDARY = "error_boundary";
    /// Category: implementation patterns.
    CATEGORY_IMPLEMENTATION = "implementation";
    /// Category: PMAT (process maturity).
    CATEGORY_PMAT = "pmat";
    /// Category: security checks.
    CATEGORY_SECURITY = "security";
}

// ============================================================================
// Validator Category Names (tag aliases + unique validators)
// ============================================================================

/// Validator: organization rules.
pub const VALIDATOR_ORGANIZATION: &str = TAG_ORGANIZATION;
/// Validator: code quality.
pub const VALIDATOR_QUALITY: &str = TAG_QUALITY;
/// Validator: SOLID principles.
pub const VALIDATOR_SOLID: &str = TAG_SOLID;
/// Validator: architecture rules.
pub const VALIDATOR_ARCHITECTURE: &str = TAG_ARCHITECTURE;
/// Validator: naming conventions.
pub const VALIDATOR_NAMING: &str = TAG_NAMING;
/// Validator: documentation rules.
pub const VALIDATOR_DOCUMENTATION: &str = TAG_DOCUMENTATION;

define_str_consts! {
    /// Validator: dependency analysis.
    VALIDATOR_DEPENDENCY = "dependency";
    /// Validator: refactoring detection.
    VALIDATOR_REFACTORING = "refactoring";
    /// Validator: design patterns.
    VALIDATOR_DESIGN_PATTERNS = "design_patterns";
    /// Validator: KISS principle.
    VALIDATOR_KISS = "kiss";
    /// Validator: test quality.
    VALIDATOR_TESTS = "tests";
    /// Validator: async patterns.
    VALIDATOR_ASYNC_PATTERNS = "async_patterns";
    /// Validator: error boundary.
    VALIDATOR_ERROR_BOUNDARY = "error_boundary";
    /// Validator: performance rules.
    VALIDATOR_PERFORMANCE = "performance";
    /// Validator: implementation rules.
    VALIDATOR_IMPLEMENTATION = "implementation";
    /// Validator: PMAT maturity.
    VALIDATOR_PMAT = "pmat";
    /// Validator: clean architecture.
    VALIDATOR_CLEAN_ARCHITECTURE = "clean_architecture";
    /// Validator: declarative rules.
    VALIDATOR_DECLARATIVE = "declarative_rules";
    /// Validator: hygiene checks.
    VALIDATOR_HYGIENE = "hygiene";
    /// Validator: pattern compliance.
    VALIDATOR_PATTERN = "pattern";
    /// Validator: port/adapter compliance.
    VALIDATOR_PORT_ADAPTER = "port_adapter";
    /// Validator: configuration quality.
    VALIDATOR_CONFIG_QUALITY = "config_quality";
    /// Validator: SSOT invariants.
    VALIDATOR_SSOT = "ssot";
    /// Validator: visibility check.
    VALIDATOR_VISIBILITY = "visibility";
    /// Validator: layer flow.
    VALIDATOR_LAYER_FLOW = "layer_flow";
    /// Validator: test quality.
    VALIDATOR_TEST_QUALITY = "test_quality";
}
