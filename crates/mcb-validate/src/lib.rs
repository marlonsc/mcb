//! Architecture Validation for MCP Context Browser
//!
//! This crate provides comprehensive validation of workspace crates against:
//! - Clean Architecture principles (dependency direction)
//! - Code quality standards (no unwrap/expect in production)
//! - Professional patterns (DI, async traits, error types)
//! - Test hygiene (no inline tests)
//! - Documentation completeness
//! - Naming conventions
//! - SOLID principles (SRP, OCP, LSP, ISP, DIP)
//! - Code organization (constants centralization, file placement)
//!
//! # Multi-Directory Validation
//!
//! Validation can scan multiple source directories (e.g., workspace crates + extra src/ trees):
//!
//! ```
//! use mcb_validate::{GenericReporter, ValidationConfig, ValidatorRegistry};
//!
//! let tmp = tempfile::tempdir().unwrap();
//! let config = ValidationConfig::new(tmp.path())
//!     .with_exclude_pattern("target/");
//!
//! let registry = ValidatorRegistry::standard_for(&config.workspace_root);
//! let violations = registry.validate_all(&config).unwrap();
//! let report = GenericReporter::create_report(&violations, config.workspace_root.clone());
//! assert!(report.summary.total_violations >= 0);
//! ```

// === Centralized Constants ===
pub mod constants;

// === Centralized Thresholds (Phase 2 DRY) ===
pub mod thresholds;

// Traits
/// Core traits for the validation system
pub mod traits;

/// Common macros for validation layer
#[macro_use]
pub mod macros;

/// Violation runtime types (field formatting, file path extraction).
pub mod violation_macro;

pub mod generic_reporter;
pub mod reporter;
pub mod run_context;
/// Validator implementations
pub mod validators;

// === Configuration System (Phase 5) ===
pub mod config;
pub mod scan;

// === Rule Registry (Phase 3) ===
pub mod embedded_rules;
pub mod engines;
pub mod rules;

// === Pattern Registry (YAML-driven patterns) ===
pub mod pattern_registry;

// === Rule Filtering System (Phase 6) ===
pub mod filters;

// === Linter Integration (Phase 1 - Pure Rust Pipeline) ===
pub mod linters;

// === AST Analysis (Phase 2 - Pure Rust Pipeline) ===
pub mod ast;

// === New Fact Extractor & Graph (Modernization) ===
pub mod extractor;
pub mod graph;

// === Metrics Analysis (Phase 4 - Complexity Metrics) ===
pub mod metrics;

// === Duplication Detection (Phase 5 - Clone Detection) ===
pub mod duplication;

// === Centralized Utilities ===
pub mod utils;

// === New Validators (using new system) ===
// Moved to validators module

// === Validators ===
// Moved to validators module

use std::path::{Path, PathBuf};

// Re-export AST module types (RCA-based)
pub use ast::{
    AstDecoder, AstNode, AstParseResult, AstQuery, AstQueryBuilder, AstQueryPatterns, AstViolation,
    Position, QueryCondition, Span, UnwrapDetection, UnwrapDetector,
};
// Re-export RCA types for direct usage (NO wrappers)
pub use ast::{Callback, LANG, Node, ParserTrait, Search, action, find, guess_language};
// New validators for PMAT integration
pub use validators::async_patterns::{AsyncPatternValidator, AsyncViolation};
// Re-export new validators
pub use validators::clean_architecture::{CleanArchitectureValidator, CleanArchitectureViolation};
// Re-export configuration system
pub use config::{
    ArchitectureRulesConfig, FileConfig, GeneralConfig, OrganizationRulesConfig,
    QualityRulesConfig, RulesConfig, SolidRulesConfig, ValidatorsConfig,
};
pub use validators::config_quality::{ConfigQualityValidator, ConfigQualityViolation};
// Re-export validators
pub use embedded_rules::EmbeddedRules;
pub use validators::dependency::{DependencyValidator, DependencyViolation};
pub use validators::documentation::{DocumentationValidator, DocumentationViolation};
// Re-export rule registry and YAML system
pub use engines::{HybridRuleEngine, RuleEngineType};
pub use validators::error_boundary::{ErrorBoundaryValidator, ErrorBoundaryViolation};
// Re-export new DRY violation system
pub use generic_reporter::{GenericReport, GenericReporter, GenericSummary};
pub use mcb_domain::ports::services::validation::ViolationEntry;
pub use validators::implementation::{ImplementationQualityValidator, ImplementationViolation};
pub use validators::kiss::{KissValidator, KissViolation};
pub use validators::layer_flow::{LayerFlowValidator, LayerFlowViolation};
// Re-export linter integration
pub use linters::{
    ClippyLinter, LintViolation, LinterEngine, LinterType, RuffLinter, YamlRuleExecutor,
};
// Re-export Metrics module types (Phase 4) - RCA-based
pub use metrics::{
    MetricThreshold, MetricThresholds, MetricType, MetricViolation, RcaAnalyzer,
    RcaFunctionMetrics, RcaMetrics,
};
pub use validators::naming::{NamingValidator, NamingViolation};
pub use validators::organization::{OrganizationValidator, OrganizationViolation};
pub use validators::pattern_validator::{PatternValidator, PatternViolation};
pub use validators::performance::{PerformanceValidator, PerformanceViolation};
pub use validators::pmat::{PmatValidator, PmatViolation};
pub use validators::port_adapter::{PortAdapterValidator, PortAdapterViolation};
pub use validators::quality::{QualityValidator, QualityViolation};
// Re-export ComponentType for strict directory validation
pub use run_context::{FileInventorySource, InventoryEntry, ValidationRunContext};
pub use validators::refactoring::{RefactoringValidator, RefactoringViolation};

pub use rules::templates::TemplateEngine;
pub use rules::yaml_loader::{
    AstSelector, MetricThresholdConfig, MetricsConfig, RuleFix, ValidatedRule, YamlRuleLoader,
};
pub use rules::yaml_validator::YamlRuleValidator;

use derive_more::Display;
use thiserror::Error;
pub use validators::hygiene::{HygieneValidator, HygieneViolation};
pub use validators::solid::{SolidValidator, SolidViolation};
pub use validators::test_quality::{TestQualityValidator, TestQualityViolation};
// Re-export centralized thresholds
pub use thresholds::{ValidationThresholds, thresholds};
pub use validators::declarative_validator::DeclarativeValidator;

pub use traits::{Validator, ValidatorRegistry};
pub use traits::{Violation, ViolationCategory};
pub use validators::visibility::{VisibilityValidator, VisibilityViolation};

// Re-export ValidationConfig for multi-directory support
// ValidationConfig is defined in this module

/// Result type for validation operations
pub type Result<T> = std::result::Result<T, ValidationError>;

/// Configuration for multi-directory validation
///
/// Allows scanning multiple source directories beyond the standard `crates/` directory.
/// Useful for validating additional source trees alongside workspace crates.
///
/// # Example
///
/// ```
/// use mcb_validate::ValidationConfig;
///
/// let config = ValidationConfig::new("/workspace")
///     .with_additional_path("../src")
///     .with_exclude_pattern("target/")
///     .with_exclude_pattern("tests/fixtures/");
/// ```
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Root directory of the workspace (contains Cargo.toml with workspace manifest)
    pub workspace_root: PathBuf,
    /// Additional source paths to validate.
    pub additional_src_paths: Vec<PathBuf>,
    /// Patterns to exclude from validation (e.g., "target/", "tests/")
    pub exclude_patterns: Vec<String>,
}

impl ValidationConfig {
    /// Create a new validation config for the given workspace root.
    ///
    /// The workspace root is canonicalized so that all downstream path
    /// comparisons (inventory `starts_with`, `strip_prefix`, etc.) work
    /// correctly on platforms where temp-dir symlinks differ from the
    /// canonical form (macOS `/tmp` → `/private/tmp`, Windows `\\?\`).
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let raw: PathBuf = workspace_root.into();
        let canonical = std::fs::canonicalize(&raw).unwrap_or(raw);
        Self {
            workspace_root: canonical,
            additional_src_paths: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }

    /// Add an additional source path to validate
    ///
    /// Paths can be absolute or relative to `workspace_root`.
    #[must_use]
    pub fn with_additional_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.additional_src_paths.push(path.into());
        self
    }

    /// Add an exclude pattern (files/directories matching this will be skipped)
    #[must_use]
    pub fn with_exclude_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// Check if a path should be excluded based on patterns
    #[must_use]
    pub fn should_exclude(&self, path: &Path) -> bool {
        let Some(path_str) = path.to_str() else {
            return false;
        };
        self.exclude_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern))
    }

    /// Get all source directories to validate
    ///
    /// Returns crates/ subdirectories plus any additional paths.
    ///
    /// # Errors
    ///
    /// Returns an error if the crates directory cannot be read.
    pub fn get_source_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();

        // Load file configuration to get skip_crates
        let file_config = FileConfig::load(&self.workspace_root);

        // Original crates/ scanning
        let crates_dir = self.workspace_root.join("crates");
        if crates_dir.exists() {
            for entry in std::fs::read_dir(&crates_dir)? {
                let entry = entry?;
                let path = entry.path();

                // Skip crates specified in configuration (e.g., validate crate itself, facade crates)
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                    && file_config
                        .general
                        .skip_crates
                        .iter()
                        .any(|skip| skip == file_name)
                {
                    continue;
                }

                if path.is_dir() && !self.should_exclude(&path) {
                    dirs.push(path);
                }
            }
        }

        // Additional paths from config
        for path in &self.additional_src_paths {
            let full_path = if path.is_absolute() {
                path.clone()
            } else {
                self.workspace_root.join(path)
            };
            if full_path.exists() && !self.should_exclude(&full_path) {
                dirs.push(full_path);
            }
        }

        Ok(dirs)
    }

    /// Get actual source directories to scan for Rust files
    ///
    /// For crate directories (containing `src/` subdirectory), returns `<dir>/src/`.
    ///
    /// # Errors
    ///
    /// Returns an error if source directory enumeration fails.
    pub fn get_scan_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut scan_dirs = Vec::new();

        for dir in self.get_source_dirs()? {
            let src_subdir = dir.join("src");
            if src_subdir.exists() && src_subdir.is_dir() {
                // Crate-style: has src/ subdirectory
                scan_dirs.push(src_subdir);
            }
            // Standard crate without src/ directory yet - skip (implicit continue)
        }

        Ok(scan_dirs)
    }
}

/// Validation error types
#[derive(Error, Debug)]
pub enum ValidationError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error
    #[error("Parse error in {file}: {message}")]
    Parse {
        /// Path to the file that failed to parse
        file: PathBuf,
        /// Error message
        message: String,
    },

    /// TOML parse error
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    /// YAML parse error
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid regex pattern
    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(#[from] regex::Error),

    /// Pattern not found
    #[error("Pattern not found: {0}")]
    PatternNotFound(String),

    /// Validation run context must be active
    #[error("Validation run context must be active")]
    ContextNotActive,

    /// Unknown validator name requested
    #[error("Unknown validator(s): {names}. Available: {available}")]
    UnknownValidator {
        /// Validator names that were not found
        names: String,
        /// Available validator names
        available: String,
    },

    /// Validator execution failed
    #[error("Validator '{name}' failed: {message}")]
    ValidatorFailed {
        /// Validator name
        name: String,
        /// Failure description
        message: String,
        /// Original error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Severity level for violations
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Display,
)]
pub enum Severity {
    /// Error severity
    #[display("ERROR")]
    Error,
    /// Warning severity
    #[display("WARNING")]
    Warning,
    /// Info severity
    #[display("INFO")]
    Info,
}

/// Component type for strict directory validation
///
/// Used to categorize code components by their architectural role,
/// enabling strict enforcement of where each type should reside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Display)]
pub enum ComponentType {
    /// Domain port trait (interface definition)
    #[display("Port")]
    Port,
    /// Domain entity with identity
    #[display("Entity")]
    Entity,
    /// Domain value object (immutable)
    #[display("ValueObject")]
    ValueObject,
    /// Domain service interface
    #[display("DomainService")]
    DomainService,
    /// Infrastructure adapter implementation
    #[display("Adapter")]
    Adapter,
    /// Repository implementation
    #[display("Repository")]
    Repository,
    /// Server/transport layer handler
    #[display("Handler")]
    Handler,
    /// Configuration type
    #[display("Config")]
    Config,
    /// Factory for creating components
    #[display("Factory")]
    Factory,
    /// DI module definition
    #[display("DiModule")]
    DiModule,
}

/// Get the workspace root from the current directory
#[must_use]
pub fn find_workspace_root() -> Option<PathBuf> {
    let current = std::env::current_dir().ok()?;
    find_workspace_root_from(&current)
}

/// Find workspace root starting from a given path
#[must_use]
pub fn find_workspace_root_from(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists()
            && let Ok(content) = std::fs::read_to_string(&cargo_toml)
            && content.contains("[workspace]")
        {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}
