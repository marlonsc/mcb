//! File-based Configuration
//!
//! Loads validation configuration from `.mcb-validate.toml` files,
//! providing project-specific rule customization.
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

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::Severity;
use crate::thresholds::{
    MAX_FILE_LINES, MAX_FUNCTION_LINES, MAX_FUNCTION_PARAMS, MAX_IMPL_METHODS, MAX_MATCH_ARMS,
    MAX_TRAIT_METHODS,
};

/// Root configuration loaded from `.mcb-validate.toml`
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FileConfig {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// Rule-specific configuration
    #[serde(default)]
    pub rules: RulesConfig,

    /// Validator enable/disable flags
    #[serde(default)]
    pub validators: ValidatorsConfig,
}

impl FileConfig {
    /// Load configuration from a file
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from workspace root, or return defaults
    ///
    /// Looks for `.mcb-validate.toml` in the workspace root directory.
    pub fn load_or_default(workspace_root: impl Into<PathBuf>) -> Self {
        let root = workspace_root.into();
        let config_path = root.join(".mcb-validate.toml");

        if config_path.exists() {
            match Self::load(&config_path) {
                Ok(mut config) => {
                    // Override workspace_root with the actual path
                    config.general.workspace_root = Some(root);
                    config
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to load {}: {}. Using defaults.",
                        config_path.display(),
                        e
                    );
                    Self::default_for(root)
                }
            }
        } else {
            Self::default_for(root)
        }
    }

    /// Create default configuration for a workspace
    pub fn default_for(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            general: GeneralConfig {
                workspace_root: Some(workspace_root.into()),
                ..Default::default()
            },
            ..Default::default()
        }
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
#[derive(Debug, Clone, Deserialize, Default)]
pub struct GeneralConfig {
    /// Workspace root path (auto-detected if not set)
    pub workspace_root: Option<PathBuf>,

    /// Patterns to exclude from validation
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Path to the rules directory
    #[serde(default = "default_rules_path")]
    pub rules_path: PathBuf,

    /// Additional source paths to validate (beyond crates/)
    #[serde(default)]
    pub additional_src_paths: Vec<PathBuf>,

    /// Output format (human, json, ci)
    #[serde(default = "default_output_format")]
    pub output_format: String,
}

fn default_output_format() -> String {
    "human".to_string()
}

fn default_rules_path() -> PathBuf {
    PathBuf::from("crates/mcb-validate/rules")
}

/// Rule-specific configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RulesConfig {
    /// Architecture validation rules
    #[serde(default)]
    pub architecture: ArchitectureRulesConfig,

    /// Code quality rules
    #[serde(default)]
    pub quality: QualityRulesConfig,

    /// Organization rules
    #[serde(default)]
    pub organization: OrganizationRulesConfig,

    /// SOLID principle rules
    #[serde(default)]
    pub solid: SolidRulesConfig,

    /// Visibility validation rules
    #[serde(default)]
    pub visibility: VisibilityRulesConfig,

    /// Layer flow validation rules
    #[serde(default)]
    pub layer_flow: LayerFlowRulesConfig,

    /// Port/Adapter validation rules
    #[serde(default)]
    pub port_adapter: PortAdapterRulesConfig,

    /// Clean Architecture rules
    #[serde(default)]
    pub clean_architecture: CleanArchitectureRulesConfig,

    /// Naming rules
    #[serde(default)]
    pub naming: NamingRulesConfig,

    /// KISS rules
    #[serde(default)]
    pub kiss: KISSRulesConfig,

    /// Refactoring rules
    #[serde(default)]
    pub refactoring: RefactoringRulesConfig,

    /// Performance rules
    #[serde(default)]
    pub performance: PerformanceRulesConfig,

    /// Pattern rules
    #[serde(default)]
    pub patterns: PatternRulesConfig,

    /// Test Quality rules
    #[serde(default)]
    pub test_quality: TestQualityRulesConfig,

    /// Implementation rules
    #[serde(default)]
    pub implementation: ImplementationRulesConfig,
}

/// Architecture validation rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ArchitectureRulesConfig {
    /// Whether architecture validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Default severity for architecture violations
    #[serde(default = "default_error_severity")]
    pub severity: Severity,

    /// Layer boundary rules
    #[serde(default)]
    pub layer_boundaries: LayerBoundariesConfig,
}

impl Default for ArchitectureRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Error,
            layer_boundaries: LayerBoundariesConfig::default(),
        }
    }
}

/// Layer boundary configuration
#[derive(Debug, Clone, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct LayerBoundariesConfig {
    /// Allowed internal dependencies for domain layer
    #[serde(default)]
    pub domain_internal_deps: Vec<String>,

    /// Allowed internal dependencies for application layer
    #[serde(default = "default_app_deps")]
    pub application_internal_deps: Vec<String>,

    /// Allowed internal dependencies for providers layer
    #[serde(default = "default_providers_deps")]
    pub providers_internal_deps: Vec<String>,

    /// Allowed internal dependencies for infrastructure layer
    #[serde(default = "default_infra_deps")]
    pub infrastructure_internal_deps: Vec<String>,

    /// Allowed internal dependencies for server layer
    #[serde(default = "default_server_deps")]
    pub server_internal_deps: Vec<String>,
}

fn default_app_deps() -> Vec<String> {
    vec!["mcb-domain".to_string()]
}

fn default_providers_deps() -> Vec<String> {
    vec!["mcb-domain".to_string(), "mcb-application".to_string()]
}

fn default_infra_deps() -> Vec<String> {
    vec![
        "mcb-domain".to_string(),
        "mcb-application".to_string(),
        "mcb-providers".to_string(),
    ]
}

fn default_server_deps() -> Vec<String> {
    vec![
        "mcb-domain".to_string(),
        "mcb-application".to_string(),
        "mcb-infrastructure".to_string(),
    ]
}

impl Default for LayerBoundariesConfig {
    fn default() -> Self {
        Self {
            domain_internal_deps: vec![],
            application_internal_deps: default_app_deps(),
            providers_internal_deps: default_providers_deps(),
            infrastructure_internal_deps: default_infra_deps(),
            server_internal_deps: default_server_deps(),
        }
    }
}

/// Code quality rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct QualityRulesConfig {
    /// Whether quality validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum lines per file
    #[serde(default = "default_max_file_lines")]
    pub max_file_lines: usize,

    /// Maximum lines per function
    #[serde(default = "default_max_function_lines")]
    pub max_function_lines: usize,

    /// Allow unwrap in test code
    #[serde(default = "default_true")]
    pub allow_unwrap_in_tests: bool,

    /// Allow expect with message (vs raw unwrap)
    #[serde(default = "default_true")]
    pub allow_expect_with_message: bool,

    /// Files/patterns exempt from unwrap/expect checks
    #[serde(default)]
    pub exempt_patterns: Vec<String>,

    /// Paths excluded from quality checks
    #[serde(default)]
    pub excluded_paths: Vec<String>,
}

fn default_max_file_lines() -> usize {
    MAX_FILE_LINES
}

fn default_max_function_lines() -> usize {
    MAX_FUNCTION_LINES
}

impl Default for QualityRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_file_lines: MAX_FILE_LINES,
            max_function_lines: MAX_FUNCTION_LINES,
            allow_unwrap_in_tests: true,
            allow_expect_with_message: true,
            exempt_patterns: vec![],
            excluded_paths: vec![
                "mcb-providers/src/vector_store/".to_string(),
                "mcb-providers/src/embedding/".to_string(),
            ],
        }
    }
}

/// Organization rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationRulesConfig {
    /// Whether organization validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Magic numbers allowed (e.g., common sizes)
    #[serde(default = "default_magic_allowlist")]
    pub magic_number_allowlist: Vec<i64>,

    /// Strict directory structure enforcement
    #[serde(default = "default_true")]
    pub strict_directory_structure: bool,
}

fn default_magic_allowlist() -> Vec<i64> {
    vec![
        0, 1, 2, 10, 100, 1000, // Common constants
        16384, 32768, 65536, // Buffer sizes
        86400, 3600, 60, // Time constants
    ]
}

impl Default for OrganizationRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            magic_number_allowlist: default_magic_allowlist(),
            strict_directory_structure: true,
        }
    }
}

/// SOLID principles rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct SolidRulesConfig {
    /// Whether SOLID validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum methods per trait (ISP)
    #[serde(default = "default_max_trait_methods")]
    pub max_trait_methods: usize,

    /// Maximum methods per impl block (SRP)
    #[serde(default = "default_max_impl_methods")]
    pub max_impl_methods: usize,

    /// Maximum match arms before suggesting polymorphism
    #[serde(default = "default_max_match_arms")]
    pub max_match_arms: usize,

    /// Maximum parameters per function
    #[serde(default = "default_max_params")]
    pub max_function_params: usize,
}

fn default_max_trait_methods() -> usize {
    MAX_TRAIT_METHODS
}

fn default_max_impl_methods() -> usize {
    MAX_IMPL_METHODS
}

fn default_max_match_arms() -> usize {
    MAX_MATCH_ARMS
}

fn default_max_params() -> usize {
    MAX_FUNCTION_PARAMS
}

impl Default for SolidRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_trait_methods: MAX_TRAIT_METHODS,
            max_impl_methods: MAX_IMPL_METHODS,
            max_match_arms: MAX_MATCH_ARMS,
            max_function_params: MAX_FUNCTION_PARAMS,
        }
    }
}

/// Visibility rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct VisibilityRulesConfig {
    /// Whether visibility validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Directories containing internal helpers (should use pub(crate))
    #[serde(default)]
    pub internal_dirs: Vec<String>,

    /// Items exempted from visibility checks
    #[serde(default)]
    pub exempted_items: Vec<String>,

    /// Patterns for utility modules to check for excessive pub items
    #[serde(default)]
    pub utility_module_patterns: Vec<String>,

    /// Threshold for pub count in utility modules
    #[serde(default = "default_pub_count_threshold")]
    pub pub_count_threshold: usize,

    /// List of crates to scan for visibility rules
    #[serde(default = "default_scan_crates")]
    pub scan_crates: Vec<String>,
}

fn default_pub_count_threshold() -> usize {
    3
}

fn default_scan_crates() -> Vec<String> {
    vec![
        "mcb-infrastructure".to_string(),
        "mcb-providers".to_string(),
        "mcb-server".to_string(),
        "mcb-application".to_string(),
        "mcb-domain".to_string(),
    ]
}

impl Default for VisibilityRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            internal_dirs: vec![],
            exempted_items: vec![],
            utility_module_patterns: vec![],
            pub_count_threshold: 3,
            scan_crates: default_scan_crates(),
        }
    }
}

/// Layer Flow rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LayerFlowRulesConfig {
    /// Whether layer flow validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Map of source crate -> list of forbidden dependency crates
    #[serde(default)]
    pub forbidden_dependencies: std::collections::HashMap<String, Vec<String>>,

    /// List of crates to check for circular dependencies
    #[serde(default)]
    pub circular_dependency_check_crates: Vec<String>,
}

impl Default for LayerFlowRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            forbidden_dependencies: std::collections::HashMap::new(),
            circular_dependency_check_crates: vec![],
        }
    }
}

/// Port/Adapter rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct PortAdapterRulesConfig {
    /// Whether port/adapter validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum methods allowed in a port trait
    #[serde(default = "default_max_port_methods")]
    pub max_port_methods: usize,

    /// Suffixes that identify adapter implementations
    #[serde(default)]
    pub adapter_suffixes: Vec<String>,

    /// Directory where ports are defined
    #[serde(default = "default_ports_dir")]
    pub ports_dir: String,

    /// Directory where providers (adapters) are defined
    #[serde(default = "default_providers_dir")]
    pub providers_dir: String,
}

fn default_max_port_methods() -> usize {
    10
}

fn default_ports_dir() -> String {
    "crates/mcb-application/src/ports".to_string()
}

fn default_providers_dir() -> String {
    "crates/mcb-providers/src".to_string()
}

impl Default for PortAdapterRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_port_methods: 10,
            adapter_suffixes: vec![],
            ports_dir: default_ports_dir(),
            providers_dir: default_providers_dir(),
        }
    }
}

/// Clean Architecture rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CleanArchitectureRulesConfig {
    /// Whether clean architecture validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    // Paths
    #[serde(default = "default_server_path")]
    pub server_path: String,
    #[serde(default = "default_handlers_path")]
    pub handlers_path: String,
    #[serde(default = "default_domain_path")]
    pub domain_path: String,
    #[serde(default = "default_entities_path")]
    pub entities_path: String,
    #[serde(default = "default_vo_path")]
    pub vo_path: String,
    #[serde(default = "default_infra_path")]
    pub infrastructure_path: String,
    #[serde(default = "default_app_path")]
    pub application_path: String,
    #[serde(default = "default_ports_providers_path")]
    pub ports_providers_path: String,

    // Patterns
    #[serde(default)]
    pub identity_skip_suffixes: Vec<String>,
    #[serde(default)]
    pub allowed_mutable_prefixes: Vec<String>,
    #[serde(default)]
    pub composition_root_skip_patterns: Vec<String>,
}

fn default_server_path() -> String {
    "crates/mcb-server".to_string()
}
fn default_handlers_path() -> String {
    "crates/mcb-server/src/handlers".to_string()
}
fn default_domain_path() -> String {
    "crates/mcb-domain".to_string()
}
fn default_entities_path() -> String {
    "crates/mcb-domain/src/entities".to_string()
}
fn default_vo_path() -> String {
    "crates/mcb-domain/src/value_objects".to_string()
}
fn default_infra_path() -> String {
    "crates/mcb-infrastructure".to_string()
}
fn default_app_path() -> String {
    "crates/mcb-application".to_string()
}
fn default_ports_providers_path() -> String {
    "crates/mcb-application/src/ports/providers".to_string()
}

impl Default for CleanArchitectureRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            server_path: default_server_path(),
            handlers_path: default_handlers_path(),
            domain_path: default_domain_path(),
            entities_path: default_entities_path(),
            vo_path: default_vo_path(),
            infrastructure_path: default_infra_path(),
            application_path: default_app_path(),
            ports_providers_path: default_ports_providers_path(),
            identity_skip_suffixes: vec![
                "Builder".to_string(),
                "Options".to_string(),
                "Config".to_string(),
                "Changes".to_string(),
            ],
            allowed_mutable_prefixes: vec![
                "set_".to_string(),
                "add_".to_string(),
                "remove_".to_string(),
                "clear_".to_string(),
                "reset_".to_string(),
            ],
            composition_root_skip_patterns: vec!["/di/".to_string()],
        }
    }
}

/// Naming rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct NamingRulesConfig {
    /// Whether naming validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Target crate for server handlers (e.g., "mcb-server")
    #[serde(default = "default_server_crate")]
    pub server_crate: String,

    /// Target crate for domain interfaces (e.g., "mcb-domain")
    #[serde(default = "default_domain_crate")]
    pub domain_crate: String,

    /// Target crate for infrastructure defaults (e.g., "mcb-infrastructure")
    #[serde(default = "default_infra_crate")]
    pub infrastructure_crate: String,
}

fn default_server_crate() -> String {
    "mcb-server".to_string()
}
fn default_domain_crate() -> String {
    "mcb-domain".to_string()
}
fn default_infra_crate() -> String {
    "mcb-infrastructure".to_string()
}

impl Default for NamingRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            server_crate: default_server_crate(),
            domain_crate: default_domain_crate(),
            infrastructure_crate: default_infra_crate(),
        }
    }
}

/// KISS rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct KISSRulesConfig {
    /// Whether KISS validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Crates excluded from KISS checks
    #[serde(default)]
    pub excluded_crates: Vec<String>,
}

impl Default for KISSRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            excluded_crates: vec!["mcb-validate".to_string(), "mcb-providers".to_string()],
        }
    }
}

/// Refactoring rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RefactoringRulesConfig {
    /// Whether refactoring validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Crates excluded from refactoring checks
    #[serde(default)]
    pub excluded_crates: Vec<String>,

    /// Generic type names to ignore (e.g. "Error", "Result")
    #[serde(default = "default_generic_type_names")]
    pub generic_type_names: Vec<String>,

    /// Utility types to ignore (e.g. "DateTime", "Uuid")
    #[serde(default = "default_utility_types")]
    pub utility_types: Vec<String>,

    /// Files to skip for refactoring checks
    #[serde(default = "default_refactoring_skip_files")]
    pub skip_files: Vec<String>,

    /// Directory patterns to skip for refactoring checks
    #[serde(default = "default_refactoring_skip_dir_patterns")]
    pub skip_dir_patterns: Vec<String>,

    /// Known pairs of crates involved in migration
    #[serde(default = "default_known_migration_pairs")]
    pub known_migration_pairs: Vec<Vec<String>>,
}

fn default_generic_type_names() -> Vec<String> {
    vec![
        "Error".to_string(),
        "Result".to_string(),
        "Config".to_string(),
        "Id".to_string(),
        "Builder".to_string(),
    ]
}

fn default_utility_types() -> Vec<String> {
    vec![
        "DateTime".to_string(),
        "Uuid".to_string(),
        "Duration".to_string(),
    ]
}

fn default_refactoring_skip_files() -> Vec<String> {
    vec![
        "mod".to_string(),
        "lib".to_string(),
        "main".to_string(),
        "types".to_string(),
        "errors".to_string(),
    ]
}

fn default_refactoring_skip_dir_patterns() -> Vec<String> {
    vec![
        "/tests/".to_string(),
        "/examples/".to_string(),
        "/benches/".to_string(),
    ]
}

fn default_known_migration_pairs() -> Vec<Vec<String>> {
    vec![
        vec!["mcb-core".to_string(), "mcb-domain".to_string()],
        vec!["mcb-core".to_string(), "mcb-infrastructure".to_string()],
    ]
}

impl Default for RefactoringRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            excluded_crates: vec!["mcb-validate".to_string()],
            generic_type_names: default_generic_type_names(),
            utility_types: default_utility_types(),
            skip_files: default_refactoring_skip_files(),
            skip_dir_patterns: default_refactoring_skip_dir_patterns(),
            known_migration_pairs: default_known_migration_pairs(),
        }
    }
}

/// Performance rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct PerformanceRulesConfig {
    /// Whether performance validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Crates excluded from performance checks
    #[serde(default)]
    pub excluded_crates: Vec<String>,
}

impl Default for PerformanceRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            excluded_crates: vec!["mcb-providers".to_string()],
        }
    }
}

/// Pattern rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct PatternRulesConfig {
    /// Whether pattern validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Crates excluded from pattern checks
    #[serde(default)]
    pub excluded_crates: Vec<String>,

    /// Regex pattern for Arc detection
    #[serde(default = "default_arc_pattern")]
    pub arc_pattern: String,

    /// Concrete types allowed in DI
    #[serde(default = "default_allowed_concrete_types")]
    pub allowed_concrete_types: Vec<String>,

    /// Trait suffixes that indicate a provider
    #[serde(default = "default_provider_trait_suffixes")]
    pub provider_trait_suffixes: Vec<String>,

    /// Crates excluded specifically from result type validation
    #[serde(default = "default_result_check_excluded_crates")]
    pub result_check_excluded_crates: Vec<String>,
}

fn default_arc_pattern() -> String {
    r"Arc<([A-Z][a-zA-Z0-9_]*)>".to_string()
}

fn default_allowed_concrete_types() -> Vec<String> {
    vec![
        "String".to_string(),
        "Vec".to_string(),
        "HashMap".to_string(),
        "RwLock".to_string(),
        "Mutex".to_string(),
    ]
}

fn default_provider_trait_suffixes() -> Vec<String> {
    vec![
        "Provider".to_string(),
        "Repository".to_string(),
        "Service".to_string(),
        "UseCase".to_string(),
    ]
}

fn default_result_check_excluded_crates() -> Vec<String> {
    vec!["mcb-providers".to_string()]
}

impl Default for PatternRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            excluded_crates: vec!["mcb-validate".to_string()],
            arc_pattern: default_arc_pattern(),
            allowed_concrete_types: default_allowed_concrete_types(),
            provider_trait_suffixes: default_provider_trait_suffixes(),
            result_check_excluded_crates: default_result_check_excluded_crates(),
        }
    }
}

/// Test Quality rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct TestQualityRulesConfig {
    /// Whether test quality validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Paths excluded from test quality checks
    #[serde(default)]
    pub excluded_paths: Vec<String>,
}

impl Default for TestQualityRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            excluded_paths: vec!["mcb-validate/src/".to_string()],
        }
    }
}

/// Implementation rules configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ImplementationRulesConfig {
    /// Whether implementation validation is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Crates excluded from implementation checks
    #[serde(default)]
    pub excluded_crates: Vec<String>,
}

impl Default for ImplementationRulesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            excluded_crates: Vec::new(),
        }
    }
}

/// Validator enable/disable flags
#[derive(Debug, Clone, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct ValidatorsConfig {
    /// Enable dependency validation
    #[serde(default = "default_true")]
    pub dependency: bool,
    /// Enable organization validation
    #[serde(default = "default_true")]
    pub organization: bool,
    /// Enable quality validation
    #[serde(default = "default_true")]
    pub quality: bool,
    /// Enable SOLID validation
    #[serde(default = "default_true")]
    pub solid: bool,
    /// Enable architecture validation
    #[serde(default = "default_true")]
    pub architecture: bool,
    /// Enable refactoring validation
    #[serde(default = "default_true")]
    pub refactoring: bool,
    /// Enable naming validation
    #[serde(default = "default_true")]
    pub naming: bool,
    /// Enable documentation validation
    #[serde(default = "default_true")]
    pub documentation: bool,
    /// Enable patterns validation
    #[serde(default = "default_true")]
    pub patterns: bool,
    /// Enable KISS validation
    #[serde(default = "default_true")]
    pub kiss: bool,
    /// Enable tests validation
    #[serde(default = "default_true")]
    pub tests: bool,
    /// Enable async patterns validation
    #[serde(default = "default_true")]
    pub async_patterns: bool,
    /// Enable error boundary validation
    #[serde(default = "default_true")]
    pub error_boundary: bool,
    /// Enable performance validation
    #[serde(default = "default_true")]
    pub performance: bool,
    /// Enable implementation validation
    #[serde(default = "default_true")]
    pub implementation: bool,
    /// Enable PMAT validation
    #[serde(default = "default_true")]
    pub pmat: bool,
    /// Enable clean architecture validation
    #[serde(default = "default_true")]
    pub clean_architecture: bool,
}

impl Default for ValidatorsConfig {
    fn default() -> Self {
        Self {
            dependency: true,
            organization: true,
            quality: true,
            solid: true,
            architecture: true,
            refactoring: true,
            naming: true,
            documentation: true,
            patterns: true,
            kiss: true,
            tests: true,
            async_patterns: true,
            error_boundary: true,
            performance: true,
            implementation: true,
            pmat: true,
            clean_architecture: true,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_error_severity() -> Severity {
    Severity::Error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FileConfig::default();
        assert!(config.validators.quality);
        assert!(config.validators.architecture);
        assert_eq!(config.rules.quality.max_file_lines, 500);
    }

    #[test]
    fn test_is_validator_enabled() {
        let config = FileConfig::default();
        assert!(config.is_validator_enabled("quality"));
        assert!(config.is_validator_enabled("architecture"));
        assert!(config.is_validator_enabled("unknown_validator"));
    }

    #[test]
    fn test_load_from_toml() {
        let toml_content = r#"
            [general]
            exclude_patterns = ["target/"]

            [rules.quality]
            max_file_lines = 300
            allow_unwrap_in_tests = false

            [validators]
            documentation = false
        "#;

        let config: FileConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.general.exclude_patterns, vec!["target/"]);
        assert_eq!(config.rules.quality.max_file_lines, 300);
        assert!(!config.rules.quality.allow_unwrap_in_tests);
        assert!(!config.validators.documentation);
        assert!(config.validators.quality); // Default true
    }

    #[test]
    fn test_layer_boundaries_defaults() {
        let config = LayerBoundariesConfig::default();
        assert!(config.domain_internal_deps.is_empty());
        assert_eq!(config.application_internal_deps, vec!["mcb-domain"]);
        assert!(
            config
                .server_internal_deps
                .contains(&"mcb-infrastructure".to_string())
        );
        assert!(
            !config
                .server_internal_deps
                .contains(&"mcb-providers".to_string())
        );
    }

    #[test]
    fn test_magic_number_allowlist() {
        let config = OrganizationRulesConfig::default();
        assert!(config.magic_number_allowlist.contains(&0));
        assert!(config.magic_number_allowlist.contains(&86400));
        assert!(config.magic_number_allowlist.contains(&65536));
    }
}
