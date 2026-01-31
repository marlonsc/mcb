//! Validation Provider Port
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

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ============================================================================
// Re-export types from services for consistency
// ============================================================================

// Note: ValidationReport and ViolationEntry are defined in ports/services.rs
// We re-export them here for convenience when implementing ValidationProvider
pub use crate::ports::services::{
    ComplexityReport, FunctionComplexity, RuleInfo, ValidationReport, ViolationEntry,
};

// ============================================================================
// Provider-specific types
// ============================================================================

/// Information about a validator available in a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Validator ID (e.g., "clean_architecture", "solid", "quality")
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
// Provider Trait
// ============================================================================

/// Pluggable Validation Provider
///
/// Defines the contract for validation engines that can analyze code for
/// various quality and architecture issues. Each provider can implement
/// different validation strategies and rule sets.
///
/// # Example
///
/// ```no_run
/// use mcb_domain::ports::providers::{ValidationProvider, ValidationOptions};
/// use std::sync::Arc;
/// use std::path::Path;
///
/// async fn validate_codebase(provider: Arc<dyn ValidationProvider>) {
///     // List available validators
///     let validators = provider.list_validators();
///     println!("Available validators: {:?}", validators.iter().map(|v| &v.id).collect::<Vec<_>>());
///
///     // Run validation
///     let options = ValidationOptions {
///         validators: Some(vec!["clean_architecture".into()]),
///         severity_filter: Some("warning".into()),
///         ..Default::default()
///     };
///
///     let report = provider.validate(Path::new("."), options).await.unwrap();
///     println!("Found {} violations", report.total_violations);
/// }
/// ```
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

// ============================================================================
// Null Implementation (for testing and fallback)
// ============================================================================

/// Null validation provider for testing or when validation feature is disabled
///
/// Returns an empty report with all checks passed. Useful for testing
/// and as a fallback when no validation provider is configured.
pub struct NullValidationProvider;

impl NullValidationProvider {
    /// Create a new null validation provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullValidationProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationProvider for NullValidationProvider {
    fn provider_name(&self) -> &str {
        "null"
    }

    fn description(&self) -> &str {
        "Null validation provider (no-op)"
    }

    fn list_validators(&self) -> Vec<ValidatorInfo> {
        Vec::new()
    }

    fn get_rules(&self, _category: Option<&str>) -> Vec<RuleInfo> {
        Vec::new()
    }

    async fn validate(
        &self,
        _workspace_root: &Path,
        _options: ValidationOptions,
    ) -> Result<ValidationReport> {
        Ok(ValidationReport {
            total_violations: 0,
            errors: 0,
            warnings: 0,
            infos: 0,
            violations: Vec::new(),
            passed: true,
        })
    }

    async fn validate_file(
        &self,
        _file_path: &Path,
        _options: ValidationOptions,
    ) -> Result<ValidationReport> {
        Ok(ValidationReport {
            total_violations: 0,
            errors: 0,
            warnings: 0,
            infos: 0,
            violations: Vec::new(),
            passed: true,
        })
    }

    fn can_validate(&self, _path: &Path) -> bool {
        false
    }

    fn supported_extensions(&self) -> &[&str] {
        &[]
    }
}
