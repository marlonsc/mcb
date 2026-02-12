//! Architecture Validation for MCP Context Browser
//!
//! This crate provides comprehensive validation of workspace crates against:
//! - Clean Architecture principles (dependency direction)
//! - Code quality standards (no unwrap/expect in production)
//! - Professional patterns (DI, async traits, error types)
//! - Test organization (no inline tests)
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
//! let config = ValidationConfig::new("/workspace")
//!     .with_additional_path("../extra-src")
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

// === New DRY Violation System (Phase 3 Refactoring) ===
pub mod violation_trait;
#[macro_use]
pub mod violation_macro;
pub mod declarative_validator;
pub mod generic_reporter;
/// Declarative registration macros used by validator composition.
pub mod macros;
pub mod reporter;
pub mod validator_trait;

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

// === New Validators (using new system) ===
pub mod clean_architecture;
pub mod config_quality;
pub mod layer_flow;
pub mod port_adapter;
pub mod test_quality;
pub mod visibility;

// === Validators ===
pub mod async_patterns;
pub mod dependency;
pub mod documentation;
pub mod error_boundary;
pub mod implementation;
pub mod kiss;
pub mod naming;
pub mod organization;
pub mod pattern_validator;
pub mod performance;
pub mod pmat;
pub(crate) mod pmat_native;
pub mod quality;
pub mod refactoring;
pub mod solid;
pub mod tests_org;

use std::path::{Path, PathBuf};

// Re-export AST module types (RCA-based)
pub use ast::{
    AstDecoder, AstNode, AstParseResult, AstQuery, AstQueryBuilder, AstQueryPatterns, AstViolation,
    Position, QueryCondition, Span, UnwrapDetection, UnwrapDetector,
};
// Re-export RCA types for direct usage (NO wrappers)
pub use ast::{Callback, LANG, Node, ParserTrait, Search, action, find, guess_language};
// New validators for PMAT integration
pub use async_patterns::{AsyncPatternValidator, AsyncViolation};
// Re-export new validators
pub use clean_architecture::{CleanArchitectureValidator, CleanArchitectureViolation};
// Re-export configuration system
pub use config::{
    ArchitectureRulesConfig, FileConfig, GeneralConfig, OrganizationRulesConfig,
    QualityRulesConfig, RulesConfig, SolidRulesConfig, ValidatorsConfig,
};
pub use config_quality::{ConfigQualityValidator, ConfigQualityViolation};
// Re-export validators
pub use dependency::{DependencyValidator, DependencyViolation};
pub use documentation::{DocumentationValidator, DocumentationViolation};
pub use embedded_rules::EmbeddedRules;
// Re-export rule registry and YAML system
pub use engines::{HybridRuleEngine, RuleEngineType};
pub use error_boundary::{ErrorBoundaryValidator, ErrorBoundaryViolation};
// Re-export new DRY violation system
pub use generic_reporter::{GenericReport, GenericReporter, GenericSummary, ViolationEntry};
pub use implementation::{ImplementationQualityValidator, ImplementationViolation};
pub use kiss::{KissValidator, KissViolation};
pub use layer_flow::{LayerFlowValidator, LayerFlowViolation};
// Re-export linter integration
pub use linters::{
    ClippyLinter, LintViolation, LinterEngine, LinterType, RuffLinter, YamlRuleExecutor,
};
// Re-export Metrics module types (Phase 4) - RCA-based
pub use metrics::{
    MetricThreshold, MetricThresholds, MetricType, MetricViolation, RcaAnalyzer,
    RcaFunctionMetrics, RcaMetrics,
};
pub use naming::{NamingValidator, NamingViolation};
pub use organization::{OrganizationValidator, OrganizationViolation};
pub use pattern_validator::{PatternValidator, PatternViolation};
pub use performance::{PerformanceValidator, PerformanceViolation};
pub use pmat::{PmatValidator, PmatViolation};
pub use port_adapter::{PortAdapterValidator, PortAdapterViolation};
pub use quality::{QualityValidator, QualityViolation};
// Re-export ComponentType for strict directory validation
pub use refactoring::{RefactoringValidator, RefactoringViolation};

pub use rules::templates::TemplateEngine;
pub use rules::yaml_loader::{
    AstSelector, MetricThresholdConfig, MetricsConfig, RuleFix, ValidatedRule, YamlRuleLoader,
};
pub use rules::yaml_validator::YamlRuleValidator;

pub use solid::{SolidValidator, SolidViolation};
pub use test_quality::{TestQualityValidator, TestQualityViolation};
pub use tests_org::{TestValidator, TestViolation};
use thiserror::Error;
// Re-export centralized thresholds
pub use declarative_validator::DeclarativeValidator;
pub use thresholds::{ValidationThresholds, thresholds};
pub use validator_trait::{Validator, ValidatorRegistry};
pub use violation_trait::{Violation, ViolationCategory};
pub use visibility::{VisibilityValidator, VisibilityViolation};

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
    /// Create a new validation config for the given workspace root
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
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
    pub fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern))
    }

    /// Get all source directories to validate
    ///
    /// Returns crates/ subdirectories plus any additional paths.
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
    InvalidRegex(String),

    /// Pattern not found
    #[error("Pattern not found: {0}")]
    PatternNotFound(String),
}

/// Severity level for violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Severity {
    /// Error severity
    Error,
    /// Warning severity
    Warning,
    /// Info severity
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warning => write!(f, "WARNING"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

/// Component type for strict directory validation
///
/// Used to categorize code components by their architectural role,
/// enabling strict enforcement of where each type should reside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ComponentType {
    /// Domain port trait (interface definition)
    Port,
    /// Domain entity with identity
    Entity,
    /// Domain value object (immutable)
    ValueObject,
    /// Domain service interface
    DomainService,
    /// Infrastructure adapter implementation
    Adapter,
    /// Repository implementation
    Repository,
    /// Server/transport layer handler
    Handler,
    /// Configuration type
    Config,
    /// Factory for creating components
    Factory,
    /// DI module definition
    DiModule,
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Port => write!(f, "Port"),
            Self::Entity => write!(f, "Entity"),
            Self::ValueObject => write!(f, "ValueObject"),
            Self::DomainService => write!(f, "DomainService"),
            Self::Adapter => write!(f, "Adapter"),
            Self::Repository => write!(f, "Repository"),
            Self::Handler => write!(f, "Handler"),
            Self::Config => write!(f, "Config"),
            Self::Factory => write!(f, "Factory"),
            Self::DiModule => write!(f, "DiModule"),
        }
    }
}

/// Get the workspace root from the current directory
pub fn find_workspace_root() -> Option<PathBuf> {
    let current = std::env::current_dir().ok()?;
    find_workspace_root_from(&current)
}

/// Find workspace root starting from a given path
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
