//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Validation Provider Registry
//!
//! Auto-registration system for validation providers using linkme distributed slices.
//! Providers register themselves via `#[linkme::distributed_slice]` and are
//! discovered at runtime.

use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for validation provider creation
///
/// Contains all configuration options that a validation provider might need.
/// Providers should use what they need and ignore the rest.
#[derive(Debug, Clone, Default)]
pub struct ValidationProviderConfig {
    /// Provider name (e.g., "mcb-validate")
    pub provider: String,
    /// Workspace root for validation scans
    pub workspace_root: Option<PathBuf>,
    /// Validator names to run (None = all)
    pub validators: Option<Vec<String>>,
    /// Minimum severity to report (error, warning, info)
    pub severity_filter: Option<String>,
    /// Path patterns to exclude from validation
    pub exclude_patterns: Option<Vec<String>>,
    /// Maximum files to validate
    pub max_files: Option<usize>,
    /// Additional provider-specific configuration
    pub extra: HashMap<String, String>,
}

crate::impl_config_builder!(ValidationProviderConfig {
    /// Set workspace root
    workspace_root: with_workspace_root(into PathBuf),
    /// Set validators to run
    validators: with_validators(Vec<String>),
    /// Set severity filter
    severity_filter: with_severity_filter(into String),
    /// Set exclude patterns
    exclude_patterns: with_exclude_patterns(Vec<String>),
    /// Set maximum files to validate
    max_files: with_max_files(usize),
});

crate::impl_registry!(
    provider_trait: crate::ports::ValidationProvider,
    config_type: ValidationProviderConfig,
    entry_type: ValidationProviderEntry,
    slice_name: VALIDATION_PROVIDERS,
    resolve_fn: resolve_validation_provider,
    list_fn: list_validation_providers
);
