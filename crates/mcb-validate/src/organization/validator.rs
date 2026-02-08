//! Organization validator implementation

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use regex::Regex;

use super::violation::OrganizationViolation;
use crate::scan::{for_each_crate_rs_path, for_each_scan_rs_path, is_test_path};
use crate::{Result, Severity, ValidationConfig};

/// Validates the structural organization and architectural compliance of the codebase.
pub struct OrganizationValidator {
    config: ValidationConfig,
}

impl OrganizationValidator {
    /// Initializes a new organization validator for the specified workspace root.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Initializes a new organization validator with a custom configuration.
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Executes all organization validation checks and returns the aggregated violations.
    pub fn validate_all(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_magic_numbers()?);
        violations.extend(self.validate_duplicate_strings()?);
        violations.extend(self.validate_file_placement()?);
        violations.extend(self.validate_trait_placement()?);
        // validate_declaration_collisions() removed - RefactoringValidator handles
        // duplicate definitions with better categorization (known migration pairs, severity)
        violations.extend(self.validate_layer_violations()?);
        // Strict CA directory and layer compliance
        violations.extend(self.validate_strict_directory()?);
        violations.extend(self.validate_domain_traits_only()?);
        Ok(violations)
    }

    /// Scans for numeric literals that should be extracted as named constants.
    pub fn validate_magic_numbers(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();

        // Pattern for numeric literals: 5+ digits (skip 4-digit numbers to reduce noise)
        let magic_pattern = Regex::new(r"\b(\d{5,})\b").expect("Invalid regex");

        // Allowed patterns (common safe numbers, powers of 2, well-known values, etc.)
        let allowed = [
            // Powers of 2
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
            // Common memory sizes (in bytes)
            "100000",
            "1000000",
            "10000000",
            "100000000",
            // Time values (seconds)
            "86400",
            "604800",
            "2592000",
            "31536000",
            // Large round numbers (often limits)
            "100000",
            "1000000",
        ];

        for_each_crate_rs_path(&self.config, |path, _src_dir, _crate_name| {
            // Skip constants.rs files (they're allowed to have numbers)
            let file_name = path.file_name().and_then(|n| n.to_str());
            if file_name.is_some_and(|n| n.contains("constant") || n.contains("config")) {
                return Ok(());
            }

            // Skip test files
            let path_str = path.to_string_lossy();
            if is_test_path(&path_str) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_test_module = false;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                // Skip comments
                if trimmed.starts_with("//") {
                    continue;
                }

                // Track test module context
                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    continue;
                }

                // Skip test modules
                if in_test_module {
                    continue;
                }

                // Skip const/static definitions (they're creating constants)
                if trimmed.starts_with("const ")
                    || trimmed.starts_with("pub const ")
                    || trimmed.starts_with("static ")
                    || trimmed.starts_with("pub static ")
                {
                    continue;
                }

                // Skip attribute macros (derive, cfg, etc.)
                if trimmed.starts_with("#[") {
                    continue;
                }

                // Skip doc comments
                if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                    continue;
                }

                // Skip assert macros (often use expected values)
                if trimmed.contains("assert") {
                    continue;
                }

                for cap in magic_pattern.captures_iter(line) {
                    let num = cap.get(1).map_or("", |m| m.as_str());

                    // Skip allowed numbers
                    if allowed.contains(&num) {
                        continue;
                    }

                    // Skip numbers that are clearly part of a constant reference
                    // e.g., _1024, SIZE_16384
                    if line.contains(&format!("_{num}")) || line.contains(&format!("{num}_")) {
                        continue;
                    }

                    // Skip underscored numbers (100_000) - they're usually constants
                    if line.contains(&format!(
                        "{}_{}",
                        &num[..num.len().min(3)],
                        &num[num.len().min(3)..]
                    )) {
                        continue;
                    }

                    violations.push(OrganizationViolation::MagicNumber {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        value: num.to_string(),
                        context: trimmed.to_string(),
                        suggestion: "Consider using a named constant".to_string(),
                        severity: Severity::Info,
                    });
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Scans for string literals duplicated across multiple files that should be centralized.
    pub fn validate_duplicate_strings(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        let mut string_occurrences: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();

        // Pattern for string literals (15+ chars to reduce noise)
        let string_pattern = Regex::new(r#""([^"\\]{15,})""#).expect("Invalid regex");

        for_each_crate_rs_path(&self.config, |path, _src_dir, _crate_name| {
            // Skip constants files (they define string constants)
            let file_name = path.file_name().and_then(|n| n.to_str());
            if file_name.is_some_and(|n| n.contains("constant")) {
                return Ok(());
            }

            // Skip test files
            let path_str = path.to_string_lossy();
            if is_test_path(&path_str) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_test_module = false;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                // Skip comments and doc strings
                if trimmed.starts_with("//") || trimmed.starts_with("#[") {
                    continue;
                }

                // Track test module context
                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    continue;
                }

                // Skip test modules
                if in_test_module {
                    continue;
                }

                // Skip const/static definitions
                if trimmed.starts_with("const ")
                    || trimmed.starts_with("pub const ")
                    || trimmed.starts_with("static ")
                    || trimmed.starts_with("pub static ")
                {
                    continue;
                }

                for cap in string_pattern.captures_iter(line) {
                    let string_val = cap.get(1).map_or("", |m| m.as_str());

                    // Skip common patterns that are OK to repeat
                    if string_val.contains("{}")           // Format strings
                        || string_val.starts_with("test_")  // Test names
                        || string_val.starts_with("Error")  // Error messages
                        || string_val.starts_with("error")
                        || string_val.starts_with("Failed")
                        || string_val.starts_with("Invalid")
                        || string_val.starts_with("Cannot")
                        || string_val.starts_with("Unable")
                        || string_val.starts_with("Missing")
                        || string_val.contains("://")       // URLs
                        || string_val.contains(".rs")       // File paths
                        || string_val.contains(".json")
                        || string_val.contains(".toml")
                        || string_val.ends_with("_id")      // ID fields
                        || string_val.ends_with("_key")     // Key fields
                        || string_val.starts_with("pub ")   // Code patterns
                        || string_val.starts_with("fn ")
                        || string_val.starts_with("let ")
                        || string_val.starts_with("CARGO_") // env!() macros
                        || string_val.contains("serde_json")// Code patterns
                        || string_val.contains(".to_string()")
                    // Method chains
                    {
                        continue;
                    }

                    string_occurrences
                        .entry(string_val.to_string())
                        .or_default()
                        .push((path.to_path_buf(), line_num + 1));
                }
            }

            Ok(())
        })?;

        // Report strings that appear in 4+ files (higher threshold)
        for (value, occurrences) in string_occurrences {
            let unique_files: HashSet<_> = occurrences.iter().map(|(f, _)| f).collect();
            if unique_files.len() >= 4 {
                violations.push(OrganizationViolation::DuplicateStringLiteral {
                    value,
                    occurrences,
                    suggestion: "Consider creating a named constant".to_string(),
                    severity: Severity::Info,
                });
            }
        }

        Ok(violations)
    }

    /// Verifies that files are located in the correct directories based on their architectural role.
    pub fn validate_file_placement(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();

        for_each_crate_rs_path(&self.config, |path, src_dir, crate_name| {
            let rel_path = path.strip_prefix(src_dir).ok();
            let path_str = path.to_string_lossy();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Check for adapter implementations in domain crate
            if crate_name.contains("domain") && path_str.contains("/adapters/") {
                violations.push(OrganizationViolation::FileInWrongLocation {
                    file: path.to_path_buf(),
                    current_location: "domain/adapters".to_string(),
                    expected_location: "infrastructure/adapters".to_string(),
                    reason: "Adapters belong in infrastructure layer".to_string(),
                    severity: Severity::Error,
                });
            }

            // Check for port definitions in infrastructure
            if crate_name.contains("infrastructure") && path_str.contains("/ports/") {
                violations.push(OrganizationViolation::FileInWrongLocation {
                    file: path.to_path_buf(),
                    current_location: "infrastructure/ports".to_string(),
                    expected_location: "domain/ports".to_string(),
                    reason: "Ports (interfaces) belong in domain layer".to_string(),
                    severity: Severity::Error,
                });
            }

            // Check for config files outside config directories
            // Exclude handler files (e.g., config_handlers.rs) - these are HTTP handlers, not config files
            if file_name.contains("config")
                && !file_name.contains("handler")
                && !path_str.contains("/config/")
                && !path_str.contains("/config.rs")
                && !path_str.contains("/admin/")
            // Admin config handlers are valid
            {
                // Allow config.rs at root level
                if rel_path.is_some_and(|p| p.components().count() > 1) {
                    violations.push(OrganizationViolation::FileInWrongLocation {
                        file: path.to_path_buf(),
                        current_location: "scattered".to_string(),
                        expected_location: "config/ directory".to_string(),
                        reason: "Configuration should be centralized".to_string(),
                        severity: Severity::Info,
                    });
                }
            }

            // Check for error handling spread across modules
            if file_name == "error.rs" {
                // Check that it's at the crate root or in a designated error module
                if rel_path.is_some_and(|p| {
                    let depth = p.components().count();
                    depth > 2 && !path_str.contains("/error/")
                }) {
                    violations.push(OrganizationViolation::FileInWrongLocation {
                        file: path.to_path_buf(),
                        current_location: "nested error.rs".to_string(),
                        expected_location: "crate root or error/ module".to_string(),
                        reason: "Error types should be centralized".to_string(),
                        severity: Severity::Info,
                    });
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Verifies that trait definitions are located in the appropriate ports directory.
    #[allow(clippy::too_many_lines)]
    pub fn validate_trait_placement(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        let trait_pattern =
            Regex::new(r"(?:pub\s+)?trait\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");

        // Traits that are OK outside ports (standard patterns)
        let allowed_traits = [
            "Debug",
            "Clone",
            "Default",
            "Display",
            "Error",
            "From",
            "Into",
            "AsRef",
            "Deref",
            "Iterator",
            "Send",
            "Sync",
            "Sized",
            "Copy",
            "Eq",
            "PartialEq",
            "Ord",
            "PartialOrd",
            "Hash",
            "Serialize",
            "Deserialize",
        ];

        // Trait suffixes that are OK in infrastructure (implementation details)
        let allowed_suffixes = [
            "Ext",       // Extension traits
            "Factory",   // Factory patterns
            "Builder",   // Builder patterns
            "Helper",    // Helper traits
            "Internal",  // Internal traits
            "Impl",      // Implementation traits
            "Adapter",   // Adapter-specific traits
            "Handler",   // Handler traits (event handlers, etc.)
            "Listener",  // Event listeners
            "Callback",  // Callback traits
            "Module",    // DI module traits
            "Component", // DI component traits
        ];

        for_each_crate_rs_path(&self.config, |path, _src_dir, crate_name| {
            // Skip domain crate (traits are allowed there)
            if crate_name.contains("domain") {
                return Ok(());
            }

            let path_str = path.to_string_lossy();

            // Skip if in ports directory (re-exports are OK)
            if path_str.contains("/ports/") {
                return Ok(());
            }

            // Skip DI modules (they often define internal traits)
            if path_str.contains("/di/") {
                return Ok(());
            }

            // Skip test files
            if is_test_path(&path_str) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_test_module = false;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                // Skip comments
                if trimmed.starts_with("//") {
                    continue;
                }

                // Track test module context
                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    continue;
                }

                // Skip traits in test modules
                if in_test_module {
                    continue;
                }

                if let Some(cap) = trait_pattern.captures(line) {
                    let trait_name = cap.get(1).map_or("", |m| m.as_str());

                    // Skip allowed traits
                    if allowed_traits.contains(&trait_name) {
                        continue;
                    }

                    // Skip internal/private traits (starts with underscore)
                    if trait_name.starts_with('_') {
                        continue;
                    }

                    // Skip traits with allowed suffixes
                    if allowed_suffixes
                        .iter()
                        .any(|suffix| trait_name.ends_with(suffix))
                    {
                        continue;
                    }

                    // Skip traits that are clearly internal (private trait declarations)
                    if trimmed.starts_with("trait ") && !trimmed.starts_with("pub trait ") {
                        continue;
                    }

                    // Infrastructure-specific provider traits that are OK outside ports
                    // These are implementation details, not domain contracts
                    let infra_provider_patterns = [
                        "CacheProvider",      // Caching is infrastructure
                        "HttpClientProvider", // HTTP client is infrastructure
                        "ConfigProvider",     // Config loading is infrastructure
                        "LogProvider",        // Logging is infrastructure
                        "MetricsProvider",    // Metrics is infrastructure
                        "TracingProvider",    // Tracing is infrastructure
                        "StorageProvider",    // Low-level storage is infra
                    ];

                    // Skip infrastructure-specific providers
                    if infra_provider_patterns.contains(&trait_name) {
                        continue;
                    }

                    // Only flag Provider/Service/Repository traits that look like ports
                    if trait_name.contains("Provider")
                        || trait_name.contains("Service")
                        || trait_name.contains("Repository")
                        || trait_name.ends_with("Interface")
                    {
                        violations.push(OrganizationViolation::TraitOutsidePorts {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            trait_name: trait_name.to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Checks for violations of Clean Architecture layer boundaries.
    ///
    /// Detects issues such as:
    /// - Server layer code directly instantiating services (bypassing DI).
    /// - Application layer code importing from the server layer (dependency inversion violation).
    pub fn validate_layer_violations(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();

        // Patterns for detecting layer violations
        let arc_new_service_pattern =
            Regex::new(r"Arc::new\s*\(\s*([A-Z][a-zA-Z0-9_]*(?:Service|Provider|Repository))::new")
                .expect("Invalid regex");
        let server_import_pattern =
            Regex::new(r"use\s+(?:crate::|super::)*server::").expect("Invalid regex");

        for_each_scan_rs_path(&self.config, true, |path, _src_dir| {
            let path_str = path.to_string_lossy();

            // Skip test files
            if is_test_path(&path_str) {
                return Ok(());
            }

            // Determine current layer
            let is_server_layer = path_str.contains("/server/");
            let is_application_layer = path_str.contains("/application/");
            let is_infrastructure_layer = path_str.contains("/infrastructure/");

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            // Track test modules to skip
            let mut in_test_module = false;
            let mut test_brace_depth: i32 = 0;
            let mut brace_depth: i32 = 0;

            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                // Track test module boundaries
                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    test_brace_depth = brace_depth;
                }

                brace_depth +=
                    i32::try_from(line.chars().filter(|c| *c == '{').count()).unwrap_or(0);
                brace_depth -=
                    i32::try_from(line.chars().filter(|c| *c == '}').count()).unwrap_or(0);

                if in_test_module && brace_depth < test_brace_depth {
                    in_test_module = false;
                }

                // Skip test modules
                if in_test_module {
                    continue;
                }

                // Skip comments
                if trimmed.starts_with("//") {
                    continue;
                }

                // Check: Server layer creating services directly
                if is_server_layer && let Some(cap) = arc_new_service_pattern.captures(line) {
                    let service_name = cap.get(1).map_or("", |m| m.as_str());

                    // Skip if it's in a builder or factory file
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if file_name.contains("builder")
                        || file_name.contains("factory")
                        || file_name.contains("bootstrap")
                    {
                        continue;
                    }

                    violations.push(OrganizationViolation::ServerCreatingServices {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        service_name: service_name.to_string(),
                        suggestion: "Use DI container to resolve services".to_string(),
                        severity: Severity::Warning,
                    });
                }

                // Check: Application layer importing from server
                if (is_application_layer || is_infrastructure_layer)
                    && server_import_pattern.is_match(line)
                    && !trimmed.contains("pub use")
                {
                    violations.push(OrganizationViolation::ApplicationImportsServer {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        import_statement: trimmed.to_string(),
                        severity: Severity::Warning,
                    });
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Enforces strict directory placement rules for specific component types (ports, adapters, handlers).
    ///
    /// Validates that:
    /// - Port traits are located in `domain/ports/`.
    /// - Adapter implementations are located in `infrastructure/adapters/`.
    /// - Handlers are located in `server/handlers/`.
    pub fn validate_strict_directory(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();

        // Patterns for detecting component types
        let port_trait_pattern = Regex::new(
            r"(?:pub\s+)?trait\s+([A-Z][a-zA-Z0-9_]*(?:Provider|Service|Repository|Interface))\s*:",
        )
        .expect("Invalid regex");
        let handler_struct_pattern =
            Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*Handler)").expect("Invalid regex");
        let adapter_impl_pattern = Regex::new(
            r"impl\s+(?:async\s+)?([A-Z][a-zA-Z0-9_]*(?:Provider|Repository))\s+for\s+([A-Z][a-zA-Z0-9_]*)"
        ).expect("Invalid regex");

        for_each_scan_rs_path(&self.config, true, |path, src_dir| {
            let is_domain_crate = src_dir.to_string_lossy().contains("domain");
            let is_infrastructure_crate = src_dir.to_string_lossy().contains("infrastructure");
            let is_server_crate = src_dir.to_string_lossy().contains("server");

            let path_str = path.to_string_lossy();

            // Skip test files
            if is_test_path(&path_str) {
                return Ok(());
            }

            // Skip mod.rs and lib.rs (aggregator files)
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name == "mod.rs" || file_name == "lib.rs" {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;

            // Check for port traits outside allowed directories
            if is_domain_crate {
                for (line_num, line) in content.lines().enumerate() {
                    if let Some(cap) = port_trait_pattern.captures(line) {
                        let trait_name = cap.get(1).map_or("", |m| m.as_str());

                        // Allowed in: ports/, domain_services/, repositories/
                        // Domain service interfaces belong in domain_services
                        // Repository interfaces belong in repositories
                        let in_allowed_dir = path_str.contains("/ports/")
                            || path_str.contains("/domain_services/")
                            || path_str.contains("/repositories/");

                        if !in_allowed_dir {
                            violations.push(OrganizationViolation::PortOutsidePorts {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                trait_name: trait_name.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }

            // Check for handlers outside allowed directories
            if is_server_crate {
                for (line_num, line) in content.lines().enumerate() {
                    if let Some(cap) = handler_struct_pattern.captures(line) {
                        let handler_name = cap.get(1).map_or("", |m| m.as_str());

                        // Allowed in: handlers/, admin/, tools/, and cross-cutting files
                        // Admin handlers belong in admin/
                        // Tool handlers belong in tools/
                        // Auth handlers are cross-cutting concerns
                        let in_allowed_location = path_str.contains("/handlers/")
                            || path_str.contains("/admin/")
                            || path_str.contains("/tools/")
                            || file_name == "auth.rs"
                            || file_name == "middleware.rs";

                        if !in_allowed_location {
                            violations.push(OrganizationViolation::HandlerOutsideHandlers {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                handler_name: handler_name.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }

            // Check for adapter implementations outside allowed directories
            if is_infrastructure_crate {
                for line in content.lines() {
                    if let Some(cap) = adapter_impl_pattern.captures(line) {
                        let _trait_name = cap.get(1).map_or("", |m| m.as_str());
                        let _impl_name = cap.get(2).map_or("", |m| m.as_str());

                        // Allowed in: adapters/, di/, and cross-cutting concern directories
                        // crypto/, cache/, health/, events/ are infrastructure cross-cutting concerns
                        let in_allowed_dir = path_str.contains("/adapters/")
                            || path_str.contains("/di/")
                            || path_str.contains("/crypto/")
                            || path_str.contains("/cache/")
                            || path_str.contains("/health/")
                            || path_str.contains("/events/")
                            || path_str.contains("/sync/")
                            || path_str.contains("/config/")
                            || path_str.contains("/infrastructure/") // Null impls for DI
                            || file_name.contains("factory")
                            || file_name.contains("bootstrap");

                        if !in_allowed_dir {
                            let current_dir = path
                                .parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_default();

                            violations.push(OrganizationViolation::StrictDirectoryViolation {
                                file: path.to_path_buf(),
                                component_type: crate::ComponentType::Adapter,
                                current_directory: current_dir,
                                expected_directory: "infrastructure/adapters/".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Verifies that the domain layer contains only trait definitions and data structures, free of implementation logic.
    ///
    /// Ensures that the domain layer remains pure and free of side effects or business logic implementation,
    /// permitting only:
    /// - Trait definitions.
    /// - Struct/enum definitions.
    /// - Constructors, accessors, and derived implementations.
    pub fn validate_domain_traits_only(&self) -> Result<Vec<OrganizationViolation>> {
        fn is_getter_method(line: &str) -> bool {
            let trimmed = line.trim();
            trimmed.contains("&self") && !trimmed.contains("&mut self")
        }
        let mut violations = Vec::new();

        // Pattern for impl blocks with methods
        let impl_block_pattern =
            Regex::new(r"impl\s+([A-Z][a-zA-Z0-9_]*)\s*\{").expect("Invalid regex");
        let method_pattern = Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(")
            .expect("Invalid regex");

        // Allowed method names (constructors, accessors, conversions, simple getters)
        let allowed_methods = [
            "new",
            "default",
            "definition", // Canonical schema factory (CA: data definition, not business logic)
            "tables",
            "fts_def",
            "indexes",
            "foreign_keys",
            "unique_constraints", // Schema builder helpers (data definition)
            "capture", // VcsContext::capture() - environment snapshot, not business logic
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
            "iter",
            "into_iter",
            // Value object utility methods
            "total_changes",
            "from_ast",
            "from_fallback",
            "directory",
            "file",
            "sorted",
            "sort_children",
            // Simple getters that start with common prefixes
        ];
        // Also allow any method starting with common prefixes (factory methods on value objects)
        // Note: These are checked inline below rather than via this array for performance
        let allowed_prefixes = [
            "from_", "into_", "as_", "to_", "get_", "is_", "has_", "with_",
        ];

        for_each_scan_rs_path(&self.config, false, |path, src_dir| {
            // Only check domain crate
            if !src_dir.to_string_lossy().contains("domain") {
                return Ok(());
            }

            let path_str = path.to_string_lossy();

            // Skip test files
            if is_test_path(&path_str) {
                return Ok(());
            }

            // Skip ports (trait definitions expected there)
            if path_str.contains("/ports/") {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            let mut in_impl_block = false;
            let mut impl_name = String::new();
            let mut brace_depth = 0;
            let mut impl_start_brace = 0;

            for (line_num, line) in lines.iter().enumerate() {
                let trimmed = line.trim();

                // Skip comments
                if trimmed.starts_with("//") {
                    continue;
                }

                // Track impl blocks
                if let Some(cap) = impl_block_pattern.captures(line)
                    && !trimmed.contains("trait ")
                {
                    in_impl_block = true;
                    impl_name = cap.get(1).map_or("", |m| m.as_str()).to_string();
                    impl_start_brace = brace_depth;
                }

                brace_depth +=
                    i32::try_from(line.chars().filter(|c| *c == '{').count()).unwrap_or(0);
                brace_depth -=
                    i32::try_from(line.chars().filter(|c| *c == '}').count()).unwrap_or(0);

                if in_impl_block && brace_depth <= impl_start_brace {
                    in_impl_block = false;
                }

                if in_impl_block && let Some(cap) = method_pattern.captures(line) {
                    let method_name = cap.get(1).map_or("", |m| m.as_str());

                    if allowed_methods.contains(&method_name) {
                        continue;
                    }

                    if allowed_prefixes.iter().any(|p| method_name.starts_with(p)) {
                        continue;
                    }

                    // Check if this is a getter method (takes &self, returns value, no side effects)
                    if is_getter_method(line) {
                        continue;
                    }

                    // This looks like business logic in domain layer
                    violations.push(OrganizationViolation::DomainLayerImplementation {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        impl_type: "method".to_string(),
                        type_name: format!("{impl_name}::{method_name}"),
                        severity: Severity::Info,
                    });
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }
}

impl_validator!(
    OrganizationValidator,
    "organization",
    "Validates code organization patterns"
);
