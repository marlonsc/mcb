//! File-based Configuration
//!
//! Loads validation configuration via figment layered providers.
//! Config files are embedded in the binary at compile time.
//!
//! # Provider Chain (later sources override earlier):
//!
//! 1. `config/mcb-validate.toml` (embedded in binary — ALL defaults)
//! 2. `config/mcb-validate-internal.toml` (filesystem — project overrides)
//! 3. Environment variables with `MCB_VALIDATE__` prefix
//!
//! # Example Configuration
//!
//! ```toml
//! [general]
//! workspace_root = "."
//! exclude_patterns = ["target/", "tests/fixtures/"]
//!
//! [rules.architecture]
//! enabled = true
//! severity = "Error"
//!
//! [rules.quality]
//! enabled = true
//! max_file_lines = 500
//! allow_unwrap_in_tests = true
//!
//! [validators]
//! dependency = true
//! organization = true
//! quality = true
//! ```

use std::path::PathBuf;

use figment::Figment;
use figment::providers::{Env, Format, Toml};
use serde::{Deserialize, Serialize};

use crate::Severity;

/// Embedded default configuration (baked into binary at compile time)
const EMBEDDED_VALIDATE_DEFAULTS: &str = include_str!("../../../../config/mcb-validate.toml");

/// Root configuration loaded via figment provider chain
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileConfig {
    /// General settings
    pub general: GeneralConfig,

    /// Rule-specific configuration
    pub rules: RulesConfig,

    /// Validator enable/disable flags
    pub validators: ValidatorsConfig,
}

impl FileConfig {
    /// Load configuration via figment layered providers.
    ///
    /// Provider chain (later sources override earlier):
    /// 1. `config/mcb-validate.toml` (embedded in binary)
    /// 2. `config/mcb-validate-internal.toml` (filesystem, project overrides)
    /// 3. Environment variables with `MCB_VALIDATE__` prefix
    ///
    /// # Panics
    ///
    /// Panics if configuration extraction fails. This is intentional —
    /// configuration errors must be caught at startup, not silently degraded.
    pub fn load(workspace_root: impl Into<PathBuf>) -> Self {
        let root = workspace_root.into();

        let figment = Figment::new()
            // Layer 1: Validator defaults (embedded in binary)
            .merge(Toml::string(EMBEDDED_VALIDATE_DEFAULTS))
            // Layer 2: Project-specific overrides (filesystem)
            .merge(Toml::file(root.join("config/mcb-validate-internal.toml")))
            // Layer 3: Runtime env overrides
            .merge(Env::prefixed("MCB_VALIDATE__").split("__").lowercase(true));

        let mut config: Self = figment
            .extract()
            .unwrap_or_else(|e| panic!("Failed to load validation config: {e}"));
        config.general.workspace_root = Some(root);
        config
    }

    /// Get the workspace root path
    pub fn workspace_root(&self) -> PathBuf {
        self.general
            .workspace_root
            .clone()
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Check if a validator is enabled
    pub fn is_validator_enabled(&self, name: &str) -> bool {
        match name {
            "dependency" => self.validators.dependency,
            "organization" => self.validators.organization,
            "quality" => self.validators.quality,
            "solid" => self.validators.solid,
            "architecture" => self.validators.architecture,
            "refactoring" => self.validators.refactoring,
            "naming" => self.validators.naming,
            "documentation" => self.validators.documentation,
            "patterns" => self.validators.patterns,
            "kiss" => self.validators.kiss,
            "tests" => self.validators.tests,
            "async_patterns" => self.validators.async_patterns,
            "error_boundary" => self.validators.error_boundary,
            "performance" => self.validators.performance,
            "implementation" => self.validators.implementation,
            "pmat" => self.validators.pmat,
            "clean_architecture" => self.validators.clean_architecture,
            _ => true, // Unknown validators enabled by default
        }
    }
}

/// General configuration settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneralConfig {
    /// Workspace root path (auto-detected if not set)
    pub workspace_root: Option<PathBuf>,

    /// Patterns to exclude from validation
    pub exclude_patterns: Vec<String>,

    /// Path to the rules directory
    pub rules_path: PathBuf,

    /// Additional source paths to validate (beyond crates/)
    pub additional_src_paths: Vec<PathBuf>,

    /// Output format (human, json, ci)
    pub output_format: String,

    /// Project prefix for internal crate names (e.g., "myapp" for myapp-domain, myapp-server)
    pub project_prefix: String,

    /// Crates to skip during validation (e.g., validate crate itself, facade crates)
    pub skip_crates: Vec<String>,

    /// Prefix for detecting internal workspace dependencies (e.g., "myapp-")
    pub internal_dep_prefix: String,
}

/// Rule-specific configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RulesConfig {
    /// Architecture validation rules
    pub architecture: ArchitectureRulesConfig,

    /// Code quality rules
    pub quality: QualityRulesConfig,

    /// Organization rules
    pub organization: OrganizationRulesConfig,

    /// SOLID principle rules
    pub solid: SolidRulesConfig,

    /// Visibility validation rules
    pub visibility: VisibilityRulesConfig,

    /// Layer flow validation rules
    pub layer_flow: LayerFlowRulesConfig,

    /// Port/Adapter validation rules
    pub port_adapter: PortAdapterRulesConfig,

    /// Clean Architecture rules
    pub clean_architecture: CleanArchitectureRulesConfig,

    /// Naming rules
    pub naming: NamingRulesConfig,

    /// KISS rules
    pub kiss: KISSRulesConfig,

    /// Refactoring rules
    pub refactoring: RefactoringRulesConfig,

    /// Performance rules
    pub performance: PerformanceRulesConfig,

    /// Pattern rules
    pub patterns: PatternRulesConfig,

    /// Test Quality rules
    pub test_quality: TestQualityRulesConfig,

    /// Implementation rules
    pub implementation: ImplementationRulesConfig,
}

/// Architecture validation rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArchitectureRulesConfig {
    /// Whether architecture validation is enabled
    pub enabled: bool,

    /// Default severity for architecture violations
    pub severity: Severity,

    /// Layer boundary rules
    pub layer_boundaries: LayerBoundariesConfig,
}

/// Layer boundary configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayerBoundariesConfig {
    /// Allowed internal dependencies for domain layer
    pub domain_internal_deps: Vec<String>,

    /// Allowed internal dependencies for application layer
    pub application_internal_deps: Vec<String>,

    /// Allowed internal dependencies for providers layer
    pub providers_internal_deps: Vec<String>,

    /// Allowed internal dependencies for infrastructure layer
    pub infrastructure_internal_deps: Vec<String>,

    /// Allowed internal dependencies for server layer
    pub server_internal_deps: Vec<String>,
}

/// Code quality rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QualityRulesConfig {
    /// Whether quality validation is enabled
    pub enabled: bool,

    /// Maximum lines per file
    pub max_file_lines: usize,

    /// Maximum lines per function
    pub max_function_lines: usize,

    /// Allow unwrap in test code
    pub allow_unwrap_in_tests: bool,

    /// Allow expect with message (vs raw unwrap)
    pub allow_expect_with_message: bool,

    /// Files/patterns exempt from unwrap/expect checks
    pub exempt_patterns: Vec<String>,

    /// Paths excluded from quality checks
    pub excluded_paths: Vec<String>,
}

/// Organization rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrganizationRulesConfig {
    /// Whether organization validation is enabled
    pub enabled: bool,

    /// Magic numbers allowed (e.g., common sizes)
    pub magic_number_allowlist: Vec<i64>,

    /// Strict directory structure enforcement
    pub strict_directory_structure: bool,
}

/// SOLID principles rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SolidRulesConfig {
    /// Whether SOLID validation is enabled
    pub enabled: bool,

    /// Maximum methods per trait (ISP)
    pub max_trait_methods: usize,

    /// Maximum methods per impl block (SRP)
    pub max_impl_methods: usize,

    /// Maximum match arms before suggesting polymorphism
    pub max_match_arms: usize,

    /// Maximum parameters per function
    pub max_function_params: usize,
}

/// Visibility rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VisibilityRulesConfig {
    /// Whether visibility validation is enabled
    pub enabled: bool,

    /// Directories containing internal helpers (should use pub(crate))
    pub internal_dirs: Vec<String>,

    /// Items exempted from visibility checks
    pub exempted_items: Vec<String>,

    /// Patterns for utility modules to check for excessive pub items
    pub utility_module_patterns: Vec<String>,

    /// Threshold for pub count in utility modules
    pub pub_count_threshold: usize,

    /// List of crates to scan for visibility rules
    pub scan_crates: Vec<String>,
}

/// Layer Flow rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayerFlowRulesConfig {
    /// Whether layer flow validation is enabled
    pub enabled: bool,

    /// Map of source crate -> list of forbidden dependency crates
    pub forbidden_dependencies: std::collections::HashMap<String, Vec<String>>,

    /// List of crates to check for circular dependencies
    pub circular_dependency_check_crates: Vec<String>,
}

/// Port/Adapter rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PortAdapterRulesConfig {
    /// Whether port/adapter validation is enabled
    pub enabled: bool,

    /// Maximum methods allowed in a port trait
    pub max_port_methods: usize,

    /// Suffixes that identify adapter implementations
    pub adapter_suffixes: Vec<String>,

    /// Directory where ports are defined
    pub ports_dir: String,

    /// Directory where providers (adapters) are defined
    pub providers_dir: String,
}

/// Clean Architecture rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CleanArchitectureRulesConfig {
    /// Whether clean architecture validation is enabled
    pub enabled: bool,

    /// Path to the server crate source directory.
    pub server_path: String,
    /// Path to the handlers directory inside the server crate.
    pub handlers_path: String,
    /// Path to the domain crate source directory.
    pub domain_path: String,
    /// Path to the entities directory inside the domain crate.
    pub entities_path: String,
    /// Path to the value objects directory inside the domain crate.
    pub vo_path: String,
    /// Path to the infrastructure crate source directory.
    pub infrastructure_path: String,
    /// Path to the application crate source directory.
    pub application_path: String,
    /// Path to the ports/providers directory inside the domain crate.
    pub ports_providers_path: String,

    /// Suffixes to skip when validating identity types.
    pub identity_skip_suffixes: Vec<String>,
    /// Allowed prefixes for mutable methods in domain entities.
    pub allowed_mutable_prefixes: Vec<String>,
    /// Path patterns to skip during composition-root analysis.
    pub composition_root_skip_patterns: Vec<String>,
}

/// Naming rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NamingRulesConfig {
    /// Whether naming validation is enabled
    pub enabled: bool,

    /// Target crate for server handlers (e.g., "mcb-server")
    pub server_crate: String,

    /// Target crate for domain interfaces (e.g., "mcb-domain")
    pub domain_crate: String,

    /// Target crate for infrastructure defaults (e.g., "mcb-infrastructure")
    pub infrastructure_crate: String,

    /// Target crate for application logic (e.g., "mcb-application")
    pub application_crate: String,

    /// Target crate for providers (e.g., "mcb-providers")
    pub providers_crate: String,

    /// Target crate for validation logic (e.g., "mcb-validate")
    pub validate_crate: String,

    /// Target crate for language support (e.g., "mcb-language-support")
    pub language_support_crate: String,

    /// Target crate for AST utilities (e.g., "mcb-ast-utils")
    pub ast_utils_crate: String,
}

/// KISS rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KISSRulesConfig {
    /// Whether KISS validation is enabled
    pub enabled: bool,

    /// Crates excluded from KISS checks
    pub excluded_crates: Vec<String>,
}

/// Refactoring rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefactoringRulesConfig {
    /// Whether refactoring validation is enabled
    pub enabled: bool,

    /// Crates excluded from refactoring checks
    pub excluded_crates: Vec<String>,

    /// Generic type names to ignore (e.g. "Error", "Result")
    pub generic_type_names: Vec<String>,

    /// Utility types to ignore (e.g. "DateTime", "Uuid")
    pub utility_types: Vec<String>,

    /// Files to skip for refactoring checks
    pub skip_files: Vec<String>,

    /// Directory patterns to skip for refactoring checks
    pub skip_dir_patterns: Vec<String>,

    /// Known pairs of crates involved in migration
    pub known_migration_pairs: Vec<Vec<String>>,
}

/// Performance rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceRulesConfig {
    /// Whether performance validation is enabled
    pub enabled: bool,

    /// Crates excluded from performance checks
    pub excluded_crates: Vec<String>,
}

/// Pattern rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PatternRulesConfig {
    /// Whether pattern validation is enabled
    pub enabled: bool,

    /// Crates excluded from pattern checks
    pub excluded_crates: Vec<String>,

    /// Regex pattern for Arc detection
    pub arc_pattern: String,

    /// Concrete types allowed in DI
    pub allowed_concrete_types: Vec<String>,

    /// Trait suffixes that indicate a provider
    pub provider_trait_suffixes: Vec<String>,

    /// Crates excluded specifically from result type validation
    pub result_check_excluded_crates: Vec<String>,
}

/// Test Quality rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestQualityRulesConfig {
    /// Whether test quality validation is enabled
    pub enabled: bool,

    /// Paths excluded from test quality checks
    pub excluded_paths: Vec<String>,
}

/// Implementation rules configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImplementationRulesConfig {
    /// Whether implementation validation is enabled
    pub enabled: bool,

    /// Crates excluded from implementation checks
    pub excluded_crates: Vec<String>,
}

/// Validator enable/disable flags
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidatorsConfig {
    /// Enable dependency validation
    pub dependency: bool,
    /// Enable organization validation
    pub organization: bool,
    /// Enable quality validation
    pub quality: bool,
    /// Enable SOLID validation
    pub solid: bool,
    /// Enable architecture validation
    pub architecture: bool,
    /// Enable refactoring validation
    pub refactoring: bool,
    /// Enable naming validation
    pub naming: bool,
    /// Enable documentation validation
    pub documentation: bool,
    /// Enable patterns validation
    pub patterns: bool,
    /// Enable KISS validation
    pub kiss: bool,
    /// Enable tests validation
    pub tests: bool,
    /// Enable async patterns validation
    pub async_patterns: bool,
    /// Enable error boundary validation
    pub error_boundary: bool,
    /// Enable performance validation
    pub performance: bool,
    /// Enable implementation validation
    pub implementation: bool,
    /// Enable PMAT validation
    pub pmat: bool,
    /// Enable clean architecture validation
    pub clean_architecture: bool,
}
