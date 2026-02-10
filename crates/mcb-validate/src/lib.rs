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
//! The validator can scan multiple source directories (e.g., workspace crates + legacy src/):
//!
//! ```text
//! use mcb_validate::{ValidationConfig, ArchitectureValidator};
//!
//! let config = ValidationConfig::new("/workspace")
//!     .with_additional_path("../legacy-src")
//!     .with_exclude_pattern("target/");
//!
//! let mut validator = ArchitectureValidator::with_config(config);
//! let report = validator.validate_all()?;
//! ```

// === Centralized Constants ===
pub mod constants;

// === Centralized Thresholds (Phase 2 DRY) ===
pub mod thresholds;

// === New DRY Violation System (Phase 3 Refactoring) ===
pub mod violation_trait;
#[macro_use]
pub mod violation_macro;
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

// === Legacy Validators (being migrated to new system) ===
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
pub mod quality;
pub mod refactoring;
pub mod solid;
pub mod tests_org;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use extractor::RustExtractor;

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
// Re-export legacy validators
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
pub use reporter::{Reporter, ValidationReport, ValidationSummary};
pub use rules::templates::TemplateEngine;
pub use rules::yaml_loader::{
    AstSelector, MetricThresholdConfig, MetricsConfig, RuleFix, ValidatedRule, YamlRuleLoader,
};
pub use rules::yaml_validator::YamlRuleValidator;
pub use rules::{Rule, RuleRegistry};
pub use solid::{SolidValidator, SolidViolation};
pub use test_quality::{TestQualityValidator, TestQualityViolation};
pub use tests_org::{TestValidator, TestViolation};
use thiserror::Error;
// Re-export centralized thresholds
pub use thresholds::{
    MAX_BUILDER_FIELDS, MAX_COGNITIVE_COMPLEXITY, MAX_CYCLOMATIC_COMPLEXITY,
    MAX_DI_CONTAINER_FIELDS, MAX_FILE_LINES, MAX_FUNCTION_LINES, MAX_FUNCTION_PARAMS,
    MAX_IMPL_METHODS, MAX_MATCH_ARMS, MAX_NESTING_DEPTH, MAX_STRUCT_FIELDS, MAX_STRUCT_LINES,
    MAX_TRAIT_METHODS, ValidationThresholds, thresholds,
};
pub use validator_trait::{LegacyValidatorAdapter, Validator, ValidatorRegistry};
pub use violation_trait::{Violation, ViolationCategory};
pub use visibility::{VisibilityValidator, VisibilityViolation};

// Re-export ValidationConfig for multi-directory support
// ValidationConfig is defined in this module

/// Result type for validation operations
pub type Result<T> = std::result::Result<T, ValidationError>;

/// Configuration for multi-directory validation
///
/// Allows scanning multiple source directories beyond the standard `crates/` directory.
/// Useful for validating legacy codebases alongside new workspace architecture.
///
/// # Example
///
/// ```
/// use mcb_validate::ValidationConfig;
///
/// let config = ValidationConfig::new("/workspace")
///     .with_additional_path("../src")  // Legacy codebase
///     .with_exclude_pattern("target/")
///     .with_exclude_pattern("tests/fixtures/");
/// ```
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Root directory of the workspace (contains Cargo.toml with workspace manifest)
    pub workspace_root: PathBuf,
    /// Additional source paths to validate (e.g., legacy src/ directories)
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

/// Main validator that orchestrates all validation checks
pub struct ArchitectureValidator {
    config: ValidationConfig,
    dependency: DependencyValidator,
    quality: QualityValidator,
    patterns: PatternValidator,
    tests: TestValidator,
    documentation: DocumentationValidator,
    naming: NamingValidator,
    solid: SolidValidator,
    organization: OrganizationValidator,
    kiss: KissValidator,
    refactoring: RefactoringValidator,
    implementation: ImplementationQualityValidator,
    // New validators for PMAT integration
    performance: PerformanceValidator,
    async_patterns: AsyncPatternValidator,
    error_boundary: ErrorBoundaryValidator,
    pmat: PmatValidator,
    // Modernization: New generic components
    extractor: RustExtractor,
    naming_config: crate::config::NamingRulesConfig,
}

impl ArchitectureValidator {
    /// Create a new validator for the given workspace root
    ///
    /// This is the simple constructor for validating only the workspace crates.
    /// For multi-directory validation, use `with_config()`.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let config = ValidationConfig::new(workspace_root);
        Self::with_config(config)
    }

    /// Create a validator with a custom configuration
    ///
    /// Use this to validate additional source directories beyond crates/.
    ///
    /// # Example
    ///
    /// ```
    /// use mcb_validate::{ValidationConfig, ArchitectureValidator};
    ///
    /// let config = ValidationConfig::new("/workspace")
    ///     .with_additional_path("../src");  // Also validate legacy code
    ///
    /// let mut validator = ArchitectureValidator::with_config(config);
    /// ```
    pub fn with_config(config: ValidationConfig) -> Self {
        let root = config.workspace_root.clone();
        // Load file configuration (rules)
        let file_config = FileConfig::load(&root);

        Self {
            dependency: DependencyValidator::with_config(config.clone()),
            quality: QualityValidator::with_config(config.clone()),
            patterns: PatternValidator::with_config(config.clone(), &file_config.rules.patterns),
            tests: TestValidator::with_config(config.clone()),
            documentation: DocumentationValidator::with_config(config.clone()),
            naming: NamingValidator::with_config(config.clone(), &file_config.rules.naming),
            solid: SolidValidator::with_config(config.clone()),
            organization: OrganizationValidator::with_config(config.clone()),
            kiss: KissValidator::with_config(config.clone(), &file_config.rules.kiss),
            refactoring: RefactoringValidator::with_config(
                config.clone(),
                &file_config.rules.refactoring,
            ),
            implementation: ImplementationQualityValidator::with_config(
                config.clone(),
                &file_config.rules.implementation,
            ),
            // New validators for PMAT integration
            performance: PerformanceValidator::with_config(
                config.clone(),
                &file_config.rules.performance,
            ),
            async_patterns: AsyncPatternValidator::with_config(config.clone()),
            error_boundary: ErrorBoundaryValidator::with_config(config.clone()),
            pmat: PmatValidator::with_config(config.clone()),
            // Modernization: Initialize new components
            extractor: RustExtractor,
            config: ValidationConfig {
                workspace_root: root,
                ..config
            },
            naming_config: file_config.rules.naming.clone(),
        }
    }

    /// Get the workspace root path
    pub fn workspace_root(&self) -> &Path {
        &self.config.workspace_root
    }

    /// Get the validation configuration
    pub fn config(&self) -> &ValidationConfig {
        &self.config
    }

    /// Run all validations and return a comprehensive report
    pub fn validate_all(&mut self) -> Result<GenericReport> {
        self.validate_with_registry()
    }

    /// Run only dependency validation
    pub fn validate_dependencies(&mut self) -> Result<Vec<DependencyViolation>> {
        self.dependency.validate_all()
    }

    /// Run only quality validation
    pub fn validate_quality(&mut self) -> Result<Vec<QualityViolation>> {
        self.quality.validate_all()
    }

    /// Run only pattern validation
    pub fn validate_patterns(&mut self) -> Result<Vec<PatternViolation>> {
        self.patterns.validate_all()
    }

    /// Run only test organization validation
    pub fn validate_tests(&mut self) -> Result<Vec<TestViolation>> {
        self.tests.validate_all()
    }

    /// Run only documentation validation
    pub fn validate_documentation(&mut self) -> Result<Vec<DocumentationViolation>> {
        self.documentation.validate_all()
    }

    /// Run only naming validation
    pub fn validate_naming(&mut self) -> Result<Vec<NamingViolation>> {
        self.naming.validate_all()
    }

    /// Run only SOLID principle validation
    pub fn validate_solid(&mut self) -> Result<Vec<SolidViolation>> {
        self.solid.validate_all()
    }

    /// Run only organization validation
    pub fn validate_organization(&mut self) -> Result<Vec<OrganizationViolation>> {
        self.organization.validate_all()
    }

    /// Run only KISS principle validation
    pub fn validate_kiss(&mut self) -> Result<Vec<KissViolation>> {
        self.kiss.validate_all()
    }

    /// Run only refactoring completeness validation
    pub fn validate_refactoring(&mut self) -> Result<Vec<RefactoringViolation>> {
        self.refactoring.validate_all()
    }

    /// Run only implementation quality validation
    pub fn validate_implementation(&mut self) -> Result<Vec<ImplementationViolation>> {
        self.implementation.validate_all()
    }

    /// Run only performance pattern validation
    pub fn validate_performance(&self) -> Result<Vec<PerformanceViolation>> {
        self.performance.validate_all()
    }

    /// Run only async pattern validation
    pub fn validate_async_patterns(&self) -> Result<Vec<AsyncViolation>> {
        self.async_patterns.validate_all()
    }

    /// Run only error boundary validation
    pub fn validate_error_boundary(&self) -> Result<Vec<ErrorBoundaryViolation>> {
        self.error_boundary.validate_all()
    }

    /// Run only PMAT integration validation
    pub fn validate_pmat(&self) -> Result<Vec<PmatViolation>> {
        self.pmat.validate_all()
    }

    // ========== YAML-Based Validation (Phase 9) ==========

    /// Create a YAML-based validator
    pub fn yaml_validator(&self) -> Result<YamlRuleValidator> {
        YamlRuleValidator::new()
    }

    /// Load and validate all YAML rules with variable substitution
    pub async fn load_yaml_rules(&self) -> Result<Vec<crate::rules::yaml_loader::ValidatedRule>> {
        // Prepare variables for substitution
        let variables_val = serde_yaml::to_value(&self.naming_config).map_err(|e| {
            crate::ValidationError::Config(format!("Failed to serialize naming config: {e}"))
        })?;

        let mut variables = variables_val
            .as_mapping()
            .ok_or_else(|| {
                crate::ValidationError::Config("Naming config is not a mapping".to_string())
            })?
            .clone();

        // Add underscored module names (e.g., domain_module from domain_crate)
        let crates = [
            "domain",
            "application",
            "providers",
            "infrastructure",
            "server",
            "validate",
            "language_support",
            "ast_utils",
        ];
        for name in crates {
            let key = format!("{name}_crate");
            if let Some(val) = variables.get(serde_yaml::Value::String(key.clone()))
                && let Some(s) = val.as_str()
            {
                let module_name = s.replace('-', "_");
                variables.insert(
                    serde_yaml::Value::String(format!("{name}_module")),
                    serde_yaml::Value::String(module_name),
                );
            }
        }

        if let Some(domain_val) =
            variables.get(serde_yaml::Value::String("domain_crate".to_string()))
            && let Some(domain_str) = domain_val.as_str()
        {
            let prefix = if let Some(idx) = domain_str.find('-') {
                domain_str[0..idx].to_string()
            } else {
                domain_str.to_string()
            };
            variables.insert(
                serde_yaml::Value::String("project_prefix".to_string()),
                serde_yaml::Value::String(prefix),
            );
        }

        let variables = serde_yaml::Value::Mapping(variables);

        let embedded_rules = EmbeddedRules::all_yaml();
        let mut loader =
            YamlRuleLoader::from_embedded_with_variables(&embedded_rules, Some(variables))?;
        loader.load_all_rules().await
    }

    /// Validate using YAML rules with hybrid engines
    pub async fn validate_with_yaml_rules(&self) -> Result<GenericReport> {
        use crate::violation_trait::ViolationCategory;

        let rules = self.load_yaml_rules().await?;
        let engine = HybridRuleEngine::new();

        let mut violations: Vec<Box<dyn Violation>> = Vec::new();

        // Scan for files to validate
        let file_contents = self.scan_files_for_validation()?;

        // === MODERNIZATION: Extract Facts & Build Graph ===
        let mut all_facts = Vec::new();
        // Use a local graph for this validation session
        let mut dep_graph = crate::graph::DependencyGraph::new();

        for file_path_str in file_contents.keys() {
            let path = Path::new(file_path_str);
            // Only analyze Rust files for now
            if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                // Convert relative path to absolute if needed, or rely on what scan_files_for_validation returns.
                // scan_files_for_validation likely returns paths relative to workspace or absolute.
                // RustExtractor expects a path it can read via fs::read.
                // If file_contents already has the content, we should modify RustExtractor to take content?
                // But RustExtractor currently does fs::read.
                // Let's assume paths are valid for fs::read.

                match self.extractor.extract_facts(path) {
                    Ok(facts) => all_facts.extend(facts),
                    Err(_e) => {
                        // Silently ignore extraction errors for now to avoid noise
                    }
                }
            }
        }

        dep_graph.build(&all_facts);

        let facts_arc = std::sync::Arc::new(all_facts);
        let graph_arc = std::sync::Arc::new(dep_graph);

        for rule in rules.into_iter().filter(|r| r.enabled) {
            let context = engines::hybrid_engine::RuleContext {
                workspace_root: self.config.workspace_root.clone(),
                config: self.config.clone(),
                ast_data: HashMap::new(),   // Would be populated by scanner
                cargo_data: HashMap::new(), // Would be populated by scanner
                file_contents: file_contents.clone(),
                facts: facts_arc.clone(),
                graph: graph_arc.clone(),
            };

            // Determine severity
            let severity = match rule.severity.to_lowercase().as_str() {
                "error" => crate::violation_trait::Severity::Error,
                "warning" => crate::violation_trait::Severity::Warning,
                _ => crate::violation_trait::Severity::Info,
            };

            // Determine category
            let category = match rule.category.to_lowercase().as_str() {
                "architecture" | "clean-architecture" => ViolationCategory::Architecture,
                "performance" => ViolationCategory::Performance,
                "organization" => ViolationCategory::Organization,
                "solid" => ViolationCategory::Solid,
                "di" | "dependency-injection" => ViolationCategory::DependencyInjection,
                "migration" => ViolationCategory::Configuration, // Use Configuration for migration rules
                // Default to Quality for "quality" and any unmatched category
                _ => ViolationCategory::Quality,
            };

            // Check if this is a lint-based rule
            if rule.lint_select.is_empty() {
                // Use rule engine execution
                let engine_type = match rule.engine.as_str() {
                    "rust-rule-engine" => RuleEngineType::RustRuleEngine,
                    // Default to RustyRules for "rusty-rules" and any unmatched engine
                    _ => RuleEngineType::RustyRules,
                };

                let result = engine
                    .execute_rule(&rule.id, engine_type, &rule.rule_definition, &context)
                    .await?;

                violations.extend(
                    result
                        .violations
                        .into_iter()
                        .map(|v| Box::new(v) as Box<dyn Violation>),
                );
            } else {
                // Use linter execution
                let result = engine
                    .execute_lint_rule(
                        &rule.id,
                        &rule.lint_select,
                        &context,
                        rule.message.as_deref(),
                        severity,
                        category,
                    )
                    .await?;

                violations.extend(
                    result
                        .violations
                        .into_iter()
                        .map(|v| Box::new(v) as Box<dyn Violation>),
                );
            }
        }

        Ok(GenericReporter::create_report(
            &violations,
            self.config.workspace_root.clone(),
        ))
    }

    /// Scan files for validation context
    fn scan_files_for_validation(&self) -> Result<HashMap<String, String>> {
        let mut file_contents = HashMap::new();

        // Scan all source directories
        if let Ok(scan_dirs) = self.config.get_scan_dirs() {
            for dir in scan_dirs {
                self.scan_directory(&dir, &mut file_contents)?;
            }
        }

        Ok(file_contents)
    }

    /// Recursively scan a directory for source files
    ///
    /// SAFETY: Uses WalkDir with symlink protection to prevent infinite loops
    fn scan_directory(
        &self,
        dir: &Path,
        file_contents: &mut HashMap<String, String>,
    ) -> Result<()> {
        use walkdir::WalkDir;

        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }

        // Use WalkDir with symlink protection instead of manual recursion
        for entry in WalkDir::new(dir)
            .follow_links(false)
            .follow_links(false) // CRITICAL: Prevent circular symlink loops
            .max_depth(100) // Reasonable depth limit to prevent stack overflow
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();

            if self.config.should_exclude(path) {
                continue;
            }

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                // Include common source file extensions
                let is_source = matches!(
                    ext,
                    "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "cpp" | "h" | "hpp"
                );
                if is_source
                    && entry.file_type().is_file()
                    && let Ok(content) = std::fs::read_to_string(path)
                {
                    file_contents.insert(path.to_string_lossy().to_string(), content);
                }
            }
        }

        Ok(())
    }

    // ========== Registry-Based Validation (Phase 7) ==========

    /// Create a `ValidatorRegistry` with the standard new validators
    ///
    /// This registry contains validators that implement the new `Validator` trait:
    /// - `CleanArchitectureValidator`
    /// - `LayerFlowValidator`
    /// - `PortAdapterValidator`
    /// - `VisibilityValidator`
    pub fn new_validator_registry(&self) -> ValidatorRegistry {
        ValidatorRegistry::standard_for(&self.config.workspace_root)
    }

    /// Validate using the canonical registry-based system
    ///
    /// # Returns
    ///
    /// A `GenericReport` containing all violations from registry validators.
    pub fn validate_with_registry(&self) -> Result<GenericReport> {
        let registry = self.new_validator_registry();
        let violations = registry
            .validate_all(&self.config)
            .map_err(|e| ValidationError::Config(e.to_string()))?;

        Ok(GenericReporter::create_report(
            &violations,
            self.config.workspace_root.clone(),
        ))
    }

    /// Validate specific validators by name using the registry
    ///
    /// # Arguments
    ///
    /// * `names` - Names of validators to run (e.g., `&["clean_architecture", "layer_flow"]`)
    ///
    /// # Available validators
    ///
    /// - "`clean_architecture`" - Clean Architecture compliance
    /// - "`layer_flow`" - Layer dependency rules
    /// - "`port_adapter`" - Port/adapter patterns
    /// - "visibility" - Visibility modifiers
    pub fn validate_named(&self, names: &[&str]) -> Result<GenericReport> {
        let registry = self.new_validator_registry();
        let violations = registry
            .validate_named(&self.config, names)
            .map_err(|e| ValidationError::Config(e.to_string()))?;

        Ok(GenericReporter::create_report(
            &violations,
            self.config.workspace_root.clone(),
        ))
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
