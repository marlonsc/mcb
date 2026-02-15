//! Centralized validation thresholds
//!
//! All numeric limits used by validators are defined here.
//! This provides a single source of truth for configuration values,
//! following the DRY principle.

use serde::{Deserialize, Serialize};

/// Validation thresholds (configurable)
///
/// Contains all numeric limits used by architecture validators.
/// Defaults are based on common code quality standards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationThresholds {
    // ========================================================================
    // SOLID Thresholds
    // ========================================================================
    /// Maximum methods per trait (Interface Segregation)
    pub max_trait_methods: usize,

    /// Maximum lines per struct definition
    pub max_struct_lines: usize,

    /// Maximum methods per impl block
    pub max_impl_methods: usize,

    /// Maximum arms in a match expression
    pub max_match_arms: usize,

    // ========================================================================
    // KISS Thresholds
    // ========================================================================
    /// Maximum fields per struct
    pub max_struct_fields: usize,

    /// Maximum parameters per function
    pub max_function_params: usize,

    /// Maximum fields in a builder pattern
    pub max_builder_fields: usize,

    /// Maximum fields in DI container (exception for catalog structs)
    pub max_di_container_fields: usize,

    /// Maximum nesting depth for control structures
    pub max_nesting_depth: usize,

    // ========================================================================
    // Quality Thresholds
    // ========================================================================
    /// Maximum lines per file
    pub max_file_lines: usize,

    /// Maximum lines per function
    pub max_function_lines: usize,

    // ========================================================================
    // Complexity Thresholds
    // ========================================================================
    /// Maximum cyclomatic complexity per function
    pub max_cyclomatic_complexity: u32,

    /// Maximum cognitive complexity per function
    pub max_cognitive_complexity: u32,
}

impl Default for ValidationThresholds {
    fn default() -> Self {
        Self {
            // SOLID
            max_trait_methods: 10,
            max_struct_lines: 700,
            max_impl_methods: 15,
            max_match_arms: 15,

            // KISS
            max_struct_fields: 16,
            max_function_params: 5,
            max_builder_fields: 7,
            max_di_container_fields: 25,
            max_nesting_depth: 3,

            // Quality
            max_file_lines: 500,
            max_function_lines: 50,

            // Complexity
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 15,
        }
    }
}

impl ValidationThresholds {
    /// Create thresholds with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

// ============================================================================
// Global Singleton (Thread-Safe)
// ============================================================================

use std::sync::OnceLock;

static THRESHOLDS: OnceLock<ValidationThresholds> = OnceLock::new();

/// Get the global validation thresholds
///
/// Returns a reference to the global thresholds singleton.
/// Initializes with defaults on first access.
pub fn thresholds() -> &'static ValidationThresholds {
    THRESHOLDS.get_or_init(ValidationThresholds::default)
}
