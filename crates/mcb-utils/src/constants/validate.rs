//!
//! Validation constants centralized from mcb-validate.
//!
//! All validation-specific constants: rule engine infrastructure, code analysis
//! patterns, detection patterns, and duplication analysis.

use super::values::{
    TAG_ARCHITECTURE, TAG_ASYNC, TAG_DOCUMENTATION, TAG_NAMING, TAG_ORGANIZATION, TAG_PERFORMANCE,
    TAG_QUALITY, TAG_SOLID,
};

// ============================================================================
// Common Patterns (cross-cutting, shared across many validators)
// ============================================================================

/// Marker for test module configuration attribute.
pub const CFG_TEST_MARKER: &str = "#[cfg(test)]";

/// Line comment prefix.
pub const COMMENT_PREFIX: &str = "//";

/// Doc comment prefix.
pub const DOC_COMMENT_PREFIX: &str = "///";

/// Module-level doc comment prefix.
pub const MODULE_DOC_PREFIX: &str = "//!";

/// Attribute macro prefix.
pub const ATTRIBUTE_PREFIX: &str = "#[";

/// Line prefixes that indicate a const or static declaration.
pub const CONST_DECLARATION_PREFIXES: &[&str] = &["const ", "pub const ", "static ", "pub static "];

/// Path fragments that indicate a test file or directory.
pub const TEST_PATH_PATTERNS: &[&str] = &["/tests/", "/target/", "_test.rs", "test.rs"];

/// Path fragment identifying a tests directory (used in single-pattern skip checks).
pub const TEST_DIR_FRAGMENT: &str = "/tests/";

/// Suffix identifying test source files.
pub const TEST_FILE_SUFFIX: &str = "_test.rs";

/// File stems that should be skipped in many validators.
pub const STANDARD_SKIP_FILES: &[&str] = &["lib", "mod", "main", "build"];

/// File name keywords that identify a constants file (skip in magic number checks, etc.).
pub const CONSTANTS_FILE_KEYWORDS: &[&str] = &["constant", "config"];

/// Prefix for test function names.
pub const TEST_FUNCTION_PREFIX: &str = "test_";

// --- Preview / Truncation Lengths ---

/// Short context preview (match expressions, error patterns) — 60 chars.
pub const SHORT_PREVIEW_LENGTH: usize = 60;

/// Standard context preview (async patterns, spawn context) — 80 chars.
pub const CONTEXT_PREVIEW_LENGTH: usize = 80;

// --- Search Radius Constants ---

/// Lines to search backward for enclosing function names.
pub const FUNCTION_NAME_SEARCH_LINES: usize = 10;

/// Lines to search backward for async trait attributes.
pub const ATTR_SEARCH_LINES: usize = 5;

/// Max forward offset for balanced-brace block extraction in `scan.rs`.
pub const MAX_BLOCK_SEARCH_OFFSET: usize = 20;

/// Lines to search forward for declarations after a marker.
pub const FORWARD_SEARCH_LINES: usize = 5;

// --- Rust Code Pattern Strings ---

/// Rust `fn` keyword prefix.
pub const FN_PREFIX: &str = "fn ";

/// Rust `pub fn` prefix.
pub const PUB_FN_PREFIX: &str = "pub fn ";

/// Rust `pub async fn` prefix.
pub const PUB_ASYNC_FN_PREFIX: &str = "pub async fn ";

/// Rust `let` binding prefix.
pub const LET_PREFIX: &str = "let ";

/// Rust `pub use` re-export prefix.
pub const PUB_USE_PREFIX: &str = "pub use";

/// Rust `use` import prefix.
pub const USE_PREFIX: &str = "use ";

/// Rust `async fn` keyword prefix.
pub const ASYNC_FN_PREFIX: &str = "async fn ";

/// All function declaration line prefixes (fn, pub fn, async fn, pub async fn).
pub const FN_PREFIXES: &[&str] = &[
    FN_PREFIX,
    PUB_FN_PREFIX,
    ASYNC_FN_PREFIX,
    PUB_ASYNC_FN_PREFIX,
];

/// Rust `mod ` keyword prefix.
pub const MOD_PREFIX: &str = "mod ";

// --- Control-Flow Detection ---

/// Tokens that indicate control flow when contained in a line (with spaces).
pub const CONTROL_FLOW_CONTAINS_TOKENS: &[&str] = &[" if ", "} else", " match ", " else {"];

/// Tokens that indicate control flow when a line starts with them.
pub const CONTROL_FLOW_STARTS_WITH_TOKENS: &[&str] = &["if ", "match ", "for ", "while ", "loop "];

// --- Error Handling Detection ---

/// `.unwrap()` method call pattern.
pub const UNWRAP_CALL: &str = ".unwrap()";

/// `.expect(` method call pattern.
pub const EXPECT_CALL: &str = ".expect(";

// --- Validation Hint Patterns ---

/// Prefix for inline ignore-hint comments (`mcb-validate-ignore: `).
pub const VALIDATE_IGNORE_PREFIX: &str = "mcb-validate-ignore: ";

// --- DI / Implementation Suffix Patterns ---

/// Common concrete-type suffixes that indicate a DI violation.
pub const DI_IMPL_SUFFIXES: &[&str] = &["Impl", "Implementation", "Adapter"];

/// Handler file suffix (e.g. `foo_handler.rs`).
pub const HANDLER_FILE_SUFFIX: &str = "_handler.rs";

/// Repository file name suffix.
pub const REPOSITORY_FILE_SUFFIX: &str = "_repository";

/// Service file name suffix.
pub const SERVICE_FILE_SUFFIX: &str = "_service";

/// Factory file name suffix.
pub const FACTORY_FILE_SUFFIX: &str = "_factory";

// --- Error Module Detection ---

/// Error module file name.
pub const ERROR_MODULE_FILE: &str = "error.rs";

/// Standard "error" identifier.
pub const VAL_ERROR: &str = "error";

/// Standard "warn" identifier.
pub const VAL_WARN: &str = "warn";

/// Standard "info" identifier.
pub const VAL_INFO: &str = "info";

// --- Allocation Detection ---

/// Standard collection type prefixes for allocation detection in loops.
pub const HEAP_ALLOC_PREFIXES: &[&str] = &["Vec::", "String::", "HashMap::", "HashSet::"];

// --- Workspace Crate Prefixes ---

/// MCB workspace crate name prefix.
pub const MCB_CRATE_PREFIX: &str = "mcb-";

/// MCB dependency name prefix (without hyphen).
pub const MCB_DEPENDENCY_PREFIX: &str = "mcb";

// ============================================================================
// Architecture Path Fragments
// ============================================================================

/// Path fragment identifying the handlers directory.
pub const ARCH_PATH_HANDLERS: &str = "/handlers/";

/// Path fragment identifying the services directory.
pub const ARCH_PATH_SERVICES: &str = "/services/";

/// Path fragment identifying the domain layer.
pub const ARCH_PATH_DOMAIN: &str = "/domain/";

/// Path fragment identifying the adapters directory.
pub const ADAPTERS_DIR: &str = "/adapters/";

/// Path fragment identifying the config directory.
pub const ARCH_PATH_CONFIG: &str = "/config/";

// ============================================================================
// Clean Architecture Naming and Layout
// ============================================================================

/// Domain-layer file keywords that indicate port traits.
pub const CA_DOMAIN_PROVIDER_KEYWORD: &str = "provider";

/// Domain-layer file keyword for repositories.
pub const CA_DOMAIN_REPOSITORY_KEYWORD: &str = "repository";

/// Expected directory for provider ports.
pub const CA_PORTS_PROVIDERS_DIR: &str = "/ports/providers/";

/// Expected directory for ports (general).
pub const PORTS_DIR: &str = "/ports/";

/// Expected directory for repositories.
pub const CA_REPOSITORIES_DIR: &str = "/repositories/";

/// Expected directory for repository adapters.
pub const CA_ADAPTERS_REPOSITORY_DIR: &str = "/adapters/repository/";

/// Infrastructure file name keywords for adapter files.
pub const CA_INFRA_IMPL_SUFFIX: &str = "_impl";

/// Infrastructure adapter file name keyword.
pub const CA_INFRA_ADAPTER_KEYWORD: &str = "adapter";

/// Infrastructure DI module file keyword.
pub const CA_MODULE_KEYWORD: &str = "module";

/// Expected directory for DI modules.
pub const CA_DI_DIR: &str = "/di/";

/// Server handler directories (allowed locations).
pub const CA_HANDLER_DIRS: &[&str] = &["/handlers/", "/admin/", "/tools/"];

/// Server handler file keyword.
pub const CA_HANDLER_KEYWORD: &str = "handler";

/// Special file names to skip in module naming checks.
pub const MODULE_SPECIAL_FILES: &[&str] = &["lib", "main", "build"];

/// Module file name (file stem).
pub const MODULE_FILE_NAME: &str = "mod";

// ============================================================================
// Labels (Pending-task and Stub Detection)
// ============================================================================

/// Label for pending task comments (first priority).
pub const PENDING_LABEL_TODO: &str = concat!("TO", "DO");

/// Label for fix-needed comments.
pub const PENDING_LABEL_FIXME: &str = concat!("FI", "XME");

/// Label for attention-needed comments.
pub const PENDING_LABEL_XXX: &str = concat!("X", "XX");

/// Label for workaround/shortcut comments.
pub const PENDING_LABEL_HACK: &str = concat!("HA", "CK");

/// Label for panic-stub detection (unimplemented placeholders).
pub const STUB_PANIC_LABEL: &str = concat!("panic(", "TO", "DO)");

// ============================================================================
// Default Validation Settings
// ============================================================================

/// Default cyclomatic complexity threshold.
pub const DEFAULT_COMPLEXITY_THRESHOLD: u32 = 15;

/// Default TDG score threshold (0-100, higher is worse).
pub const DEFAULT_TDG_THRESHOLD: u32 = 50;

/// Default max lines per file before triggering a size violation.
pub const DEFAULT_MAX_FILE_LINES: usize = 500;

// ============================================================================
// YAML Rule Field Names (macro-generated)
// ============================================================================

use super::define_str_consts;

define_str_consts! {
    /// YAML field: rule identifier.
    YAML_FIELD_ID = "id";
    /// YAML field: rule display name.
    YAML_FIELD_NAME = "name";
    /// YAML field: rule category.
    YAML_FIELD_CATEGORY = "category";
    /// YAML field: rule severity level.
    YAML_FIELD_SEVERITY = "severity";
    /// YAML field: rule enabled flag.
    YAML_FIELD_ENABLED = "enabled";
    /// YAML field: rule description text.
    YAML_FIELD_DESCRIPTION = "description";
    /// YAML field: rule rationale text.
    YAML_FIELD_RATIONALE = "rationale";
    /// YAML field: rule engine type.
    YAML_FIELD_ENGINE = "engine";
    /// YAML field: rule configuration block.
    YAML_FIELD_CONFIG = "config";
    /// YAML field: rule definition block.
    YAML_FIELD_RULE = "rule";
    /// YAML field: auto-fix suggestions.
    YAML_FIELD_FIXES = "fixes";
    /// YAML field: fix type.
    YAML_FIELD_FIX_TYPE = "type";
    /// YAML field: pattern match string.
    YAML_FIELD_PATTERN = "pattern";
    /// YAML field: violation message.
    YAML_FIELD_MESSAGE = "message";
    /// YAML field: lint select rules.
    YAML_FIELD_LINT_SELECT = "lint_select";
    /// YAML field: selectors block.
    YAML_FIELD_SELECTORS = "selectors";
    /// YAML field: language filter.
    YAML_FIELD_LANGUAGE = "language";
    /// YAML field: AST node type.
    YAML_FIELD_NODE_TYPE = "node_type";
    /// YAML field: AST query string.
    YAML_FIELD_AST_QUERY = "ast_query";
    /// YAML field: metrics thresholds.
    YAML_FIELD_METRICS = "metrics";
    /// YAML field: file filters.
    YAML_FIELD_FILTERS = "filters";
    /// YAML field: template base marker.
    YAML_FIELD_BASE = "_base";
    /// YAML field: template reference.
    YAML_FIELD_TEMPLATE = "_template";
    /// YAML field: rule extension marker.
    YAML_FIELD_EXTENDS = "_extends";
    /// YAML field: regex pattern.
    YAML_FIELD_REGEX = "regex";
    /// YAML field: patterns array.
    YAML_FIELD_PATTERNS = "patterns";
    /// YAML field: crate name.
    YAML_FIELD_CRATE_NAME = "crate_name";
    /// YAML field: allowed dependencies list.
    YAML_FIELD_ALLOWED_DEPS = "allowed_dependencies";
    /// YAML field: rule expression.
    YAML_FIELD_EXPRESSION = "expression";
    /// YAML field: rule condition.
    YAML_FIELD_CONDITION = "condition";
    /// YAML field: rule action.
    YAML_FIELD_ACTION = "action";
    /// YAML field: GRL rule definition.
    GRL = "grl";
    /// YAML field: rule definition block reference.
    YAML_FIELD_RULE_DEFINITION = "rule_definition";
}

define_str_consts! {
    // --- GRL Keywords ---
    /// GRL keyword: "when" condition block.
    GRL_KEYWORD_WHEN = "when";
    /// GRL keyword: "then" action block.
    GRL_KEYWORD_THEN = "then";
    // --- Metrics Specific Fields ---
    /// YAML field: cognitive complexity metric.
    YAML_FIELD_COGNITIVE_COMPLEXITY = "cognitive_complexity";
    /// YAML field: cyclomatic complexity metric.
    YAML_FIELD_CYCLOMATIC_COMPLEXITY = "cyclomatic_complexity";
    /// YAML field: function length metric.
    YAML_FIELD_FUNCTION_LENGTH = "function_length";
    /// YAML field: nesting depth metric.
    YAML_FIELD_NESTING_DEPTH = "nesting_depth";
    // --- Rusty Rules specific fields ---
    /// Rusty Rules field: "all" logical operation.
    RUSTY_FIELD_ALL = "all";
    /// Rusty Rules field: "any" logical operation.
    RUSTY_FIELD_ANY = "any";
    /// Rusty Rules field: "not" logical operation.
    RUSTY_FIELD_NOT = "not";
    /// Rusty Rules field: fact type identifier.
    RUSTY_FIELD_FACT_TYPE = "fact_type";
    /// Rusty Rules field: field name indicator.
    RUSTY_FIELD_FIELD = "field";
    /// Rusty Rules field: matching operator.
    RUSTY_FIELD_OPERATOR = "operator";
    /// Rusty Rules field: expected value.
    RUSTY_FIELD_VALUE = "value";
    /// Rusty Rules field: violation details.
    RUSTY_FIELD_VIOLATION = "violation";
    // --- Metrics Threshold Fields ---
    /// Metrics field: maximum threshold.
    METRICS_FIELD_MAX = "max";
    /// Metrics field: severity override.
    METRICS_FIELD_SEVERITY_OVERRIDE = "severity_override";
    // --- Rule Defaults ---
    /// Default rule name when not specified.
    DEFAULT_RULE_NAME = "Unnamed Rule";
    /// Default rule description.
    DEFAULT_RULE_DESCRIPTION = "No description provided";
    /// Default rule rationale.
    DEFAULT_RULE_RATIONALE = "No rationale provided";
    /// Default rule engine type.
    RUSTY_RULES = "rusty-rules";
    /// Default violation message for expression engine rules.
    DEFAULT_EXPR_RULE_ID = "EXPR_RULE";
    /// Default expression engine violation message.
    DEFAULT_EXPR_MESSAGE = "Expression rule violation";
    /// Default Rete engine violation message.
    DEFAULT_RETE_MESSAGE = "Rule violation detected";
    /// Default GRL rule ID.
    DEFAULT_GRL_RULE_ID = "GRL_RULE";
    /// Default violation message for rusty-rules engine.
    DEFAULT_VIOLATION_MESSAGE = "Rule violation";
    // --- Rule Engine Type Identifiers ---
    /// Rete network engine type.
    ENGINE_TYPE_RETE = "rete";
    /// Rust Rule Engine type.
    ENGINE_TYPE_RUST_RULE = "rust-rule-engine";
    /// `EvalExpr` engine type.
    ENGINE_TYPE_EVALEXPR = "evalexpr";
    /// JSON DSL engine type.
    ENGINE_TYPE_JSON_DSL = "json-dsl";
    // --- Rusty Rules Engine Defaults ---
    /// Default rule type when not specified.
    GENERIC = "generic";
    /// Default field name for condition checks.
    RUSTY_DEFAULT_FIELD = "value";
    /// Default operator for condition checks.
    RUSTY_DEFAULT_OPERATOR = "equals";
}

/// Default rule category.
pub const DEFAULT_RULE_CATEGORY: &str = TAG_QUALITY;

/// Cargo dependency condition: `not_exists`.
pub const NOT_EXISTS: &str = "not_exists";

/// File size rule condition: `exceeds_limit`.
pub const RUSTY_DEFAULT_FILE_SIZE_CONDITION: &str = "exceeds_limit";

/// Default file extension pattern for `file_size` rules.
pub const RUSTY_DEFAULT_FILE_SIZE_PATTERN: &str = ".rs";

/// Default label for custom actions.
pub const RUSTY_CUSTOM_ACTION_DEFAULT: &str = "Custom action";

/// Violation ID for cargo dependency rules.
pub const RUSTY_CARGO_DEP_VIOLATION_ID: &str = "CARGO_DEP";

/// Message when required dependency is missing.
pub const RUSTY_CARGO_DEP_MISSING_MSG: &str = "Required dependency not found";

/// Message when forbidden dependency is present.
pub const RUSTY_CARGO_DEP_FORBIDDEN_MSG: &str = "Forbidden dependency found";

/// Violation ID for AST pattern rules.
pub const RUSTY_AST_PATTERN_VIOLATION_ID: &str = "AST_PATTERN";

/// Path fragment for target directory (skip in scans).
pub const RUSTY_TARGET_DIR_FRAGMENT: &str = "/target/";

/// Rule type: `cargo_dependencies`.
pub const RUSTY_RULE_TYPE_CARGO_DEPENDENCIES: &str = "cargo_dependencies";

/// Rule type: `file_size`.
pub const RUSTY_RULE_TYPE_FILE_SIZE: &str = "file_size";

/// Rule type: `ast_pattern`.
pub const RUSTY_RULE_TYPE_AST_PATTERN: &str = "ast_pattern";

/// Condition: exists.
pub const RUSTY_CONDITION_EXISTS: &str = "exists";

// --- Linter Command Names ---

/// Ruff linter command name.
pub const LINTER_CMD_RUFF: &str = "ruff";

/// Cargo command name (for Clippy).
pub const LINTER_CMD_CARGO: &str = "cargo";

// ============================================================================
// COMMON IDENTIFIERS AND TAGS — defined in values.rs, re-exported via mod.rs
// ============================================================================

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

// ============================================================================
// Linter Integration (Clippy / Cargo)
// ============================================================================

/// Clippy rule code prefix.
pub const CLIPPY_PREFIX: &str = "clippy::";

/// Clippy CLI warning flag.
pub const CLIPPY_WARN_FLAG: &str = "-W";

/// Clippy subcommand name.
pub const CLIPPY_COMMAND: &str = "clippy";

/// Clippy JSON output format flag.
pub const CLIPPY_MESSAGE_FORMAT_JSON: &str = "--message-format=json";

/// Cargo argument separator.
pub const CARGO_ARG_SEPARATOR: &str = "--";

/// Cargo manifest filename.
pub const CARGO_TOML_FILENAME: &str = "Cargo.toml";

/// Clippy compiler-message reason string.
pub const CLIPPY_REASON_COMPILER_MESSAGE: &str = "compiler-message";

// ============================================================================
// Quality Detection (unwrap/panic)
// ============================================================================

/// Safety justification comment markers.
pub const SAFETY_COMMENT_MARKERS: &[&str] = &["// SAFETY:", "// safety:"];

/// Ignore hint keywords for unwrap/expect suppression.
pub const IGNORE_HINT_KEYWORDS: &[&str] = &["lock_poisoning_recovery"];

/// Number of lines before/after a detection to search for ignore hints.
pub const COMMENT_SEARCH_RADIUS: usize = 3;

/// Strings that indicate legitimate lock-poisoning `expect()` usage.
pub const LOCK_POISONING_STRINGS: &[&str] = &[
    "lock poisoned",
    "Lock poisoned",
    "poisoned",
    "Mutex poisoned",
];

/// Regex pattern for detecting `panic!()` macro usage.
pub const PANIC_REGEX: &str = r"panic!\s*\(";

// ============================================================================
// SOLID Detection
// ============================================================================

/// Max unrelated structs in a single file before SRP warning.
pub const MAX_UNRELATED_STRUCTS_PER_FILE: usize = 5;

/// Min string-based match arms before OCP dispatch warning.
pub const MIN_STRING_MATCH_ARMS_FOR_DISPATCH: usize = 3;

/// Min names needed for relationship check.
pub const MIN_NAMES_FOR_RELATION_CHECK: usize = 2;

/// Min shared prefix/suffix length for relationship detection.
pub const MIN_AFFIX_LENGTH: usize = 3;

/// Max shared prefix/suffix length for relationship detection.
pub const MAX_AFFIX_LENGTH: usize = 10;

/// Min word length for semantic comparison in CamelCase splitting.
pub const MIN_WORD_LENGTH_FOR_COMPARISON: usize = 4;

// ============================================================================
// KISS Detection
// ============================================================================

/// Type name suffixes that identify DI container structs (allowed more fields).
pub const DI_CONTAINER_SUFFIXES: &[&str] = &["Context", "Container", "Components", "State"];

/// Type name substrings that identify config-like structs (allowed more fields).
pub const DI_CONTAINER_CONTAINS: &[&str] = &["Config", "Settings"];

/// Minimum line distance between reported nesting violations to avoid noise.
pub const NESTING_PROXIMITY_THRESHOLD: usize = 5;

// ============================================================================
// Refactoring Detection
// ============================================================================

/// Regex pattern for detecting type definitions (struct, trait, enum).
pub const TYPE_DEFINITION_REGEX: &str =
    r"(?:pub\s+)?(?:struct|trait|enum)\s+([A-Z][a-zA-Z0-9_]*)(?:\s*<|\s*\{|\s*;|\s*\(|\s+where)";

/// Path patterns for files to skip in duplicate detection.
pub const REFACTORING_SKIP_PATTERNS: &[&str] = &["/tests/", "_test.rs", ".archived", ".bak"];

/// Crate path delimiter for extracting crate names.
pub const CRATE_PATH_DELIMITER: &str = "/crates/";

/// Type name suffixes that suggest migration in progress.
pub const MIGRATION_TYPE_SUFFIXES: &[&str] = &[
    "Provider",
    "Processor",
    "Handler",
    "Service",
    "Repository",
    "Adapter",
    "Factory",
    "Publisher",
    "Subscriber",
];

// ============================================================================
// Implementation Detection
// ============================================================================

/// Hardcoded return pattern IDs and descriptions: (`pattern_id`, description).
pub const HARDCODED_RETURN_PATTERNS: &[(&str, &str)] = &[
    ("IMPL001.return_true", "true"),
    ("IMPL001.return_false", "false"),
    ("IMPL001.return_zero", "0"),
    ("IMPL001.return_one", "1"),
    ("IMPL001.return_empty_string", "empty string"),
    ("IMPL001.return_hardcoded_string", "hardcoded string"),
];

/// File names to skip in hardcoded return detection.
pub const STUB_SKIP_FILE_KEYWORDS: &[&str] = &["null", "fake", "constants.rs"];

// ============================================================================
// Documentation Detection
// ============================================================================

/// Regex for detecting doc comments (`///`).
pub const DOC_COMMENT_REGEX: &str = r"^\s*///";

/// Regex for capturing doc comment content after `///`.
pub const DOC_COMMENT_CAPTURE_REGEX: &str = r"^\s*///(.*)";

/// Regex for detecting attributes (`#[...]`).
pub const ATTR_REGEX: &str = r"^\s*#\[";

/// Regex for detecting module-level doc comments (`//!`).
pub const MODULE_DOC_REGEX: &str = "^//!";

/// Regex for detecting `pub struct` declarations.
pub const PUB_STRUCT_REGEX: &str = r"pub\s+struct\s+([A-Z][a-zA-Z0-9_]*)";

/// Regex for detecting `pub enum` declarations.
pub const PUB_ENUM_REGEX: &str = r"pub\s+enum\s+([A-Z][a-zA-Z0-9_]*)";

/// Regex for detecting `pub trait` declarations.
pub const PUB_TRAIT_REGEX: &str = r"pub\s+trait\s+([A-Z][a-zA-Z0-9_]*)";

/// Regex for detecting `pub fn` / `pub async fn` declarations.
pub const PUB_FN_REGEX: &str = r"pub\s+(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)";

/// Regex for detecting example sections in documentation.
pub const EXAMPLE_SECTION_REGEX: &str = r"#\s*Example";

/// File names that require module-level documentation.
pub const MODULE_FILE_NAMES: &[&str] = &["lib.rs", "mod.rs"];

/// Paths identifying DI module traits (skip example checking).
pub const DI_MODULES_PATH: &str = "/di/modules/";

/// Label for struct items in violation messages.
pub const ITEM_KIND_STRUCT: &str = "struct";

/// Label for enum items in violation messages.
pub const ITEM_KIND_ENUM: &str = "enum";

/// Label for trait items in violation messages.
pub const ITEM_KIND_TRAIT: &str = "trait";

/// Label for function items in violation messages.
pub const ITEM_KIND_FUNCTION: &str = "function";

// ============================================================================
// Async Pattern Detection
// ============================================================================

/// Patterns for detecting wrong mutex types in async code: (pattern, description, suggestion).
pub const WRONG_MUTEX_PATTERNS: &[(&str, &str, &str)] = &[
    (
        r"use\s+std::sync::Mutex",
        "std::sync::Mutex import",
        "Use tokio::sync::Mutex for async code",
    ),
    (
        "std::sync::Mutex<",
        "std::sync::Mutex type",
        "Use tokio::sync::Mutex for async code",
    ),
    (
        r"use\s+std::sync::RwLock",
        "std::sync::RwLock import",
        "Use tokio::sync::RwLock for async code",
    ),
    (
        "std::sync::RwLock<",
        "std::sync::RwLock type",
        "Use tokio::sync::RwLock for async code",
    ),
];

/// Function name keywords that indicate intentional fire-and-forget spawns.
pub const BACKGROUND_FN_PATTERNS: &[&str] = &[
    "spawn",
    "background",
    "graceful",
    "shutdown",
    "start",
    "run",
    "worker",
    "daemon",
    "listener",
    "handler",
    "process",
    "new",
    "with_",
    "init",
    "create",
    "build",
];

// ============================================================================
// Performance Detection
// ============================================================================

/// Patterns for detecting Arc/Mutex overuse: (pattern, description, suggestion).
pub const ARC_MUTEX_OVERUSE_PATTERNS: &[(&str, &str, &str)] = &[
    ("Arc<Arc<", "Nested Arc<Arc<>>", "Use single Arc instead"),
    ("Mutex<bool>", "Mutex<bool>", "Use AtomicBool instead"),
    ("Mutex<usize>", "Mutex<usize>", "Use AtomicUsize instead"),
    ("Mutex<u32>", "Mutex<u32>", "Use AtomicU32 instead"),
    ("Mutex<u64>", "Mutex<u64>", "Use AtomicU64 instead"),
    ("Mutex<i32>", "Mutex<i32>", "Use AtomicI32 instead"),
    ("Mutex<i64>", "Mutex<i64>", "Use AtomicI64 instead"),
    ("RwLock<bool>", "RwLock<bool>", "Use AtomicBool instead"),
];

/// Patterns for detecting inefficient iterator usage.
pub const INEFFICIENT_ITERATOR_PATTERNS: &[(&str, &str, &str)] = &[
    (
        r"\.iter\(\)\.cloned\(\)\.take\(",
        ".iter().cloned().take()",
        "Use .iter().take().cloned() instead",
    ),
    (
        r"\.iter\(\)\.cloned\(\)\.last\(",
        ".iter().cloned().last()",
        "Use .iter().last().cloned() instead",
    ),
    (
        r#"\.collect::<Vec<String>>\(\)\.join\(\s*""\s*\)"#,
        r#".collect::<Vec<String>>().join("")"#,
        "Use .collect::<String>() instead",
    ),
    (
        r"\.repeat\(1\)",
        ".repeat(1)",
        "Use .clone() instead of .repeat(1)",
    ),
];

/// Patterns for detecting inefficient string handling.
pub const INEFFICIENT_STRING_PATTERNS: &[(&str, &str, &str)] = &[
    (
        r#"format!\s*\(\s*"\{\}"\s*,\s*\w+\s*\)"#,
        "format!(\"{}\", var)",
        "Use var.to_string() or &var instead",
    ),
    (
        r"\.to_string\(\)\.to_string\(\)",
        ".to_string().to_string()",
        "Remove redundant .to_string()",
    ),
    (
        r"\.to_owned\(\)\.to_owned\(\)",
        ".to_owned().to_owned()",
        "Remove redundant .to_owned()",
    ),
];

/// Regex pattern for detecting `.clone()` calls.
pub const CLONE_REGEX: &str = r"\.clone\(\)";

/// Regex patterns for detecting allocations in loops.
pub const LOOP_ALLOCATION_PATTERNS: &[&str] = &[
    r"Vec::new\(\)",
    r"Vec::with_capacity\(",
    r"String::new\(\)",
    r"String::with_capacity\(",
    r"HashMap::new\(\)",
    r"HashSet::new\(\)",
];

/// Maximum characters of context to include in clone-in-loop violations.
pub const CONTEXT_TRUNCATION_LENGTH: usize = CONTEXT_PREVIEW_LENGTH;

// ============================================================================
// Organization Detection
// ============================================================================

/// Regex pattern for detecting 5+ digit magic numbers.
pub const MAGIC_NUMBER_REGEX: &str = r"\b(\d{5,})\b";

/// Allowed numeric literals (powers of 2, memory sizes, time values).
pub const ALLOWED_MAGIC_NUMBERS: &[&str] = &[
    "16384",
    "32768",
    "65535",
    "65536",
    "131072",
    "262144",
    "524288",
    "1048576",
    "2097152",
    "4194304",
    "100000",
    "1000000",
    "10000000",
    "100000000",
    "86400",
    "604800",
    "2592000",
    "31536000",
];

/// Regex for extracting string literals (15+ characters).
pub const DUPLICATE_STRING_REGEX: &str = r#""([^"\\]{15,})""#;

/// Minimum number of files a string must appear in to be flagged.
pub const DUPLICATE_STRING_MIN_FILES: usize = 4;

/// Patterns in string values that are OK to repeat across files.
pub const DUPLICATE_STRING_SKIP_PATTERNS: &[&str] = &[
    "{}",
    "test_",
    "Error",
    "error",
    "Failed",
    "Invalid",
    "Cannot",
    "Unable",
    "Missing",
    "://",
    ".rs",
    ".json",
    ".toml",
    "_id",
    "_key",
    "pub ",
    "fn ",
    "let ",
    "CARGO_",
    "serde_json",
    ".to_string()",
];

/// Allowed method names in domain impl blocks.
pub const DOMAIN_ALLOWED_METHODS: &[&str] = &[
    "new",
    "default",
    "definition",
    "tables",
    "fts_def",
    "indexes",
    "foreign_keys",
    "unique_constraints",
    "from",
    "into",
    "as_ref",
    "as_mut",
    "clone",
    "fmt",
    "eq",
    "cmp",
    "hash",
    "partial_cmp",
    "is_empty",
    "len",
    "success",
    "error",
    "render",
    "iter",
    "into_iter",
    "total_changes",
    "from_ast",
    "from_fallback",
    "directory",
    "file",
    "sorted",
    "sort_children",
];

/// Allowed method name prefixes in domain impl blocks.
pub const DOMAIN_ALLOWED_PREFIXES: &[&str] = &[
    "from_", "into_", "as_", "to_", "get_", "is_", "has_", "with_",
];

/// Path fragment identifying the domain crate.
pub const DOMAIN_CRATE_PATH: &str = "domain";

/// Regex for detecting direct service instantiation via `Arc::new(Service::new`.
pub const ARC_NEW_SERVICE_REGEX: &str =
    r"Arc::new\s*\(\s*([A-Z][a-zA-Z0-9_]*(?:Service|Provider|Repository))::new";

/// Regex for detecting server-layer imports.
pub const SERVER_IMPORT_REGEX: &str = r"use\s+(?:crate::|super::)*server::";

/// Path fragment identifying the server layer.
pub const SERVER_LAYER_PATH: &str = "/server/";

/// Path fragment identifying the application layer.
pub const APPLICATION_LAYER_PATH: &str = "/application/";

/// Path fragment identifying the infrastructure layer.
pub const INFRASTRUCTURE_LAYER_PATH: &str = "/infrastructure/";

/// File names that are allowed to bypass the direct service creation rule.
pub const SERVICE_CREATION_BYPASS_FILES: &[&str] = &["builder", "factory", "bootstrap"];

// ============================================================================
// Clone Detection (Duplication)
// ============================================================================

/// Common keywords to ignore when fingerprinting (multi-language).
pub const DUPLICATION_KEYWORDS: &[&str] = &[
    // Rust
    "fn",
    "let",
    "mut",
    "const",
    "static",
    "struct",
    "enum",
    "impl",
    "trait",
    "pub",
    "mod",
    "use",
    "crate",
    "self",
    "super",
    "where",
    "async",
    "await",
    "move",
    "ref",
    "match",
    "if",
    "else",
    "loop",
    "while",
    "for",
    "in",
    "break",
    "continue",
    "return",
    "type",
    "as",
    "dyn",
    "unsafe",
    "extern",
    // Python
    "def",
    "class",
    "import",
    "from",
    "with",
    "try",
    "except",
    "finally",
    "raise",
    "pass",
    "yield",
    "lambda",
    "global",
    "nonlocal",
    "assert",
    "del",
    "True",
    "False",
    "None",
    "and",
    "or",
    "not",
    "is",
    // JavaScript / TypeScript
    "function",
    "var",
    "extends",
    "implements",
    "interface",
    "namespace",
    "module",
    "export",
    "default",
    "new",
    "delete",
    "typeof",
    "instanceof",
    "this",
    "null",
    "undefined",
    "true",
    "false",
    "void",
    "throw",
    "catch",
    "debugger",
    "switch",
    "case",
];

/// Base for the Rabin-Karp rolling hash (small prime).
pub const RABIN_KARP_BASE: u64 = 31;

/// Modulus for the Rabin-Karp rolling hash (large prime for collision resistance).
pub const RABIN_KARP_MODULUS: u64 = 1_000_000_007;

/// Placeholder for normalized identifiers in Type-2 (renamed) clone detection.
pub const NORMALIZED_IDENTIFIER: &str = "$ID";

/// Placeholder for normalized literals in Type-2 (renamed) clone detection.
pub const NORMALIZED_LITERAL: &str = "$LIT";

/// Characters classified as operators in token classification.
pub const OPERATOR_CHARS: &str = "+-*%=<>!&|^~";

/// Characters classified as punctuation in token classification.
pub const PUNCTUATION_CHARS: &str = "(){}[];:,.?";

/// Default minimum lines for a clone to be reported.
pub const DEFAULT_MIN_LINES: usize = 6;

/// Default minimum tokens for a clone to be reported.
pub const DEFAULT_MIN_TOKENS: usize = 50;

/// Default similarity threshold for duplicate detection.
pub const DEFAULT_SIMILARITY_THRESHOLD: f64 = 0.80;

/// Default maximum gap size for gapped (Type-3) clones.
pub const DEFAULT_MAX_GAP_SIZE: usize = 5;

/// Strict mode: minimum lines.
pub const STRICT_MIN_LINES: usize = 4;

/// Strict mode: minimum tokens.
pub const STRICT_MIN_TOKENS: usize = 30;

/// Strict mode: similarity threshold.
pub const STRICT_SIMILARITY_THRESHOLD: f64 = 0.90;

/// Lenient mode: minimum lines.
pub const LENIENT_MIN_LINES: usize = 10;

/// Lenient mode: minimum tokens.
pub const LENIENT_MIN_TOKENS: usize = 100;

/// Lenient mode: similarity threshold.
pub const LENIENT_SIMILARITY_THRESHOLD: f64 = 0.70;

/// Languages analyzed by default.
pub const DEFAULT_LANGUAGES: &[&str] = &["rust", "python", "javascript", "typescript"];

/// Glob patterns excluded from duplication analysis by default.
pub const DEFAULT_EXCLUDE_PATTERNS: &[&str] = &[
    "**/target/**",
    "**/node_modules/**",
    "**/.git/**",
    "**/vendor/**",
];
