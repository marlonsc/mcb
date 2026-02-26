//! Validation Provider Port
//!
//! **Documentation**: [docs/modules/domain.md](../../../../../docs/modules/domain.md#provider-ports)
//!
//! Port for pluggable code validation providers. Implementations can be
//! different validation engines (mcb-validate, clippy, ESLint, etc.) that
//! analyze code for architecture violations, code quality issues, and more.
//!
//! ## Provider Pattern
//!
//! This port follows the same pattern as [`EmbeddingProvider`] and
//! [`VectorStoreProvider`], enabling consistent provider registration,
//! factory creation, and feature-flag based compilation via linkme.
//!
//! ## Difference from ValidationServiceInterface
//!
//! - `ValidationServiceInterface` (in ports/services.rs) is an application
//!   service port that orchestrates validation across providers
//! - `ValidationProvider` (this trait) is the provider port that actual
//!   validation engines implement
//!
//! This separation allows multiple validation providers to be registered
//! and selected at runtime based on configuration.

use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;
// ============================================================================
// Re-export types from services for consistency
// ============================================================================

// Note: ValidationReport and ViolationEntry are defined in ports/services.rs
// We re-export them here for convenience when implementing ValidationProvider
pub use crate::ports::{RuleInfo, ValidationReport};

// ============================================================================
// Provider-specific types
// ============================================================================

/// Information about a validator available in a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Validator ID (e.g., "`clean_architecture`", "solid", "quality")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this validator checks
    pub description: String,
    /// Number of rules in this validator
    pub rule_count: usize,
    /// Categories of rules (e.g., ["layering", "dependencies"])
    pub categories: Vec<String>,
}

/// Options for validation operations
#[derive(Debug, Clone, Default)]
pub struct ValidationOptions {
    /// Specific validators to run (None = all)
    pub validators: Option<Vec<String>>,
    /// Minimum severity to report ("error", "warning", "info")
    pub severity_filter: Option<String>,
    /// Patterns to exclude from validation
    pub exclude_patterns: Option<Vec<String>>,
    /// Maximum files to validate (for limiting scope)
    pub max_files: Option<usize>,
    /// Whether to include suggestions in output
    pub include_suggestions: bool,
}

// ============================================================================
// Rule validator (linkme-discovered validators)
// ============================================================================

/// Request for running one or more rule validators.
#[derive(Debug, Clone)]
pub struct RuleValidatorRequest {
    /// Workspace root path
    pub workspace_root: std::path::PathBuf,
    /// Validator names to run (None = all registered)
    pub validator_names: Option<Vec<String>>,
    /// Minimum severity to report
    pub severity_filter: Option<String>,
    /// Exclude patterns
    pub exclude_patterns: Option<Vec<String>>,
}

/// Port for a single rule validator (e.g. clean_architecture, quality).
///
/// Validators are discovered via linkme in [`crate::registry::validation::VALIDATOR_ENTRIES`]
/// and built with a workspace root. Implementations live in mcb-validate.
pub trait RuleValidator: Send + Sync {
    /// Unique validator name (e.g. "clean_architecture", "quality").
    fn name(&self) -> &'static str;

    /// Run this validator and return a validation report.
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails (e.g. I/O, parse).
    fn run(
        &self,
        request: &RuleValidatorRequest,
    ) -> std::result::Result<ValidationReport, crate::error::Error>;
}

// ============================================================================
// Provider Trait
// ============================================================================

/// Pluggable Validation Provider
///
/// Defines the contract for validation engines that can analyze code for
/// various quality and architecture issues. Each provider can implement
/// different validation strategies and rule sets.
#[async_trait]
pub trait ValidationProvider: Send + Sync {
    /// Get the provider name
    ///
    /// Returns a unique identifier for this provider (e.g., "mcb-validate", "clippy")
    fn provider_name(&self) -> &str;

    /// Get provider description
    fn description(&self) -> &str;

    /// List all validators available in this provider
    ///
    /// Returns information about each validator including its ID, name,
    /// description, and the number of rules it contains.
    fn list_validators(&self) -> Vec<ValidatorInfo>;

    /// Get rules available in this provider
    ///
    /// # Arguments
    /// * `category` - Optional category filter
    ///
    /// # Returns
    /// List of rule definitions
    fn get_rules(&self, category: Option<&str>) -> Vec<RuleInfo>;

    /// Validate a workspace/directory
    ///
    /// # Arguments
    /// * `workspace_root` - Path to the workspace root directory
    /// * `options` - Validation options (validators, severity filter, etc.)
    ///
    /// # Returns
    /// Validation report with all violations found
    async fn validate(
        &self,
        workspace_root: &Path,
        options: ValidationOptions,
    ) -> Result<ValidationReport>;

    /// Validate a single file
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to validate
    /// * `options` - Validation options
    ///
    /// # Returns
    /// Validation report for the single file
    async fn validate_file(
        &self,
        file_path: &Path,
        options: ValidationOptions,
    ) -> Result<ValidationReport>;

    /// Check if a file can be validated by this provider
    ///
    /// # Arguments
    /// * `path` - Path to check
    ///
    /// # Returns
    /// true if this provider can validate the file
    fn can_validate(&self, path: &Path) -> bool;

    /// Get supported file extensions
    ///
    /// Returns the file extensions this provider can validate
    fn supported_extensions(&self) -> &[&str];
}
