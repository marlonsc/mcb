//! Code Organization Validation
//!
//! Validates code organization:
//! - Constants centralization (magic numbers, duplicate strings)
//! - Type centralization (types should be in domain layer)
//! - File placement (files in correct architectural layers)
//! - Declaration collision detection

use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Organization violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationViolation {
    /// Magic number found (should be a named constant)
    MagicNumber {
        file: PathBuf,
        line: usize,
        value: String,
        context: String,
        suggestion: String,
        severity: Severity,
    },

    /// Duplicate string literal across files
    DuplicateStringLiteral {
        value: String,
        occurrences: Vec<(PathBuf, usize)>,
        suggestion: String,
        severity: Severity,
    },

    /// Constant defined in wrong module (should be centralized)
    DecentralizedConstant {
        file: PathBuf,
        line: usize,
        constant_name: String,
        suggestion: String,
        severity: Severity,
    },

    /// Type defined in wrong layer
    TypeInWrongLayer {
        file: PathBuf,
        line: usize,
        type_name: String,
        current_layer: String,
        expected_layer: String,
        severity: Severity,
    },

    /// File in wrong architectural location
    FileInWrongLocation {
        file: PathBuf,
        current_location: String,
        expected_location: String,
        reason: String,
        severity: Severity,
    },

    /// Declaration collision (same name in multiple places)
    DeclarationCollision {
        name: String,
        locations: Vec<(PathBuf, usize, String)>, // (file, line, type)
        severity: Severity,
    },

    /// Trait defined outside ports layer
    TraitOutsidePorts {
        file: PathBuf,
        line: usize,
        trait_name: String,
        severity: Severity,
    },

    /// Provider/Adapter implementation outside infrastructure
    AdapterOutsideInfrastructure {
        file: PathBuf,
        line: usize,
        impl_name: String,
        severity: Severity,
    },

    /// Constants file too large (should be split by domain)
    ConstantsFileTooLarge {
        file: PathBuf,
        line_count: usize,
        max_allowed: usize,
        severity: Severity,
    },

    /// Common magic number pattern detected (vector dimensions, timeouts, pool sizes)
    CommonMagicNumber {
        file: PathBuf,
        line: usize,
        value: String,
        pattern_type: String,
        suggestion: String,
        severity: Severity,
    },

    /// File too large without module decomposition
    LargeFileWithoutModules {
        file: PathBuf,
        line_count: usize,
        max_allowed: usize,
        suggestion: String,
        severity: Severity,
    },

    /// Same service/type defined in multiple layers
    DualLayerDefinition {
        type_name: String,
        locations: Vec<(PathBuf, String)>, // (file, layer)
        severity: Severity,
    },

    /// Server layer creating application services directly
    ServerCreatingServices {
        file: PathBuf,
        line: usize,
        service_name: String,
        suggestion: String,
        severity: Severity,
    },

    /// Application layer importing from server
    ApplicationImportsServer {
        file: PathBuf,
        line: usize,
        import_statement: String,
        severity: Severity,
    },
}

impl OrganizationViolation {
    pub fn severity(&self) -> Severity {
        match self {
            Self::MagicNumber { severity, .. } => *severity,
            Self::DuplicateStringLiteral { severity, .. } => *severity,
            Self::DecentralizedConstant { severity, .. } => *severity,
            Self::TypeInWrongLayer { severity, .. } => *severity,
            Self::FileInWrongLocation { severity, .. } => *severity,
            Self::DeclarationCollision { severity, .. } => *severity,
            Self::TraitOutsidePorts { severity, .. } => *severity,
            Self::AdapterOutsideInfrastructure { severity, .. } => *severity,
            Self::ConstantsFileTooLarge { severity, .. } => *severity,
            Self::CommonMagicNumber { severity, .. } => *severity,
            Self::LargeFileWithoutModules { severity, .. } => *severity,
            Self::DualLayerDefinition { severity, .. } => *severity,
            Self::ServerCreatingServices { severity, .. } => *severity,
            Self::ApplicationImportsServer { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for OrganizationViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MagicNumber {
                file,
                line,
                value,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Magic number: {}:{} - {} ({})",
                    file.display(),
                    line,
                    value,
                    suggestion
                )
            }
            Self::DuplicateStringLiteral {
                value,
                occurrences,
                suggestion,
                ..
            } => {
                let locations: Vec<String> = occurrences
                    .iter()
                    .map(|(p, l)| format!("{}:{}", p.display(), l))
                    .collect();
                write!(
                    f,
                    "Duplicate string literal \"{}\": [{}] - {}",
                    value,
                    locations.join(", "),
                    suggestion
                )
            }
            Self::DecentralizedConstant {
                file,
                line,
                constant_name,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Decentralized constant: {}:{} - {} ({})",
                    file.display(),
                    line,
                    constant_name,
                    suggestion
                )
            }
            Self::TypeInWrongLayer {
                file,
                line,
                type_name,
                current_layer,
                expected_layer,
                ..
            } => {
                write!(
                    f,
                    "Type in wrong layer: {}:{} - {} is in {} but should be in {}",
                    file.display(),
                    line,
                    type_name,
                    current_layer,
                    expected_layer
                )
            }
            Self::FileInWrongLocation {
                file,
                current_location,
                expected_location,
                reason,
                ..
            } => {
                write!(
                    f,
                    "File in wrong location: {} is in {} but should be in {} ({})",
                    file.display(),
                    current_location,
                    expected_location,
                    reason
                )
            }
            Self::DeclarationCollision {
                name, locations, ..
            } => {
                let locs: Vec<String> = locations
                    .iter()
                    .map(|(p, l, t)| format!("{}:{}({})", p.display(), l, t))
                    .collect();
                write!(
                    f,
                    "Declaration collision: {} found at [{}]",
                    name,
                    locs.join(", ")
                )
            }
            Self::TraitOutsidePorts {
                file,
                line,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "Trait outside ports: {}:{} - {} should be in domain/ports",
                    file.display(),
                    line,
                    trait_name
                )
            }
            Self::AdapterOutsideInfrastructure {
                file,
                line,
                impl_name,
                ..
            } => {
                write!(
                    f,
                    "Adapter outside infrastructure: {}:{} - {} should be in infrastructure/adapters",
                    file.display(),
                    line,
                    impl_name
                )
            }
            Self::ConstantsFileTooLarge {
                file,
                line_count,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "Constants file too large: {} has {} lines (max: {}) - consider splitting by domain",
                    file.display(),
                    line_count,
                    max_allowed
                )
            }
            Self::CommonMagicNumber {
                file,
                line,
                value,
                pattern_type,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Common magic number: {}:{} - {} ({}) - {}",
                    file.display(),
                    line,
                    value,
                    pattern_type,
                    suggestion
                )
            }
            Self::LargeFileWithoutModules {
                file,
                line_count,
                max_allowed,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Large file without modules: {} has {} lines (max: {}) - {}",
                    file.display(),
                    line_count,
                    max_allowed,
                    suggestion
                )
            }
            Self::DualLayerDefinition {
                type_name,
                locations,
                ..
            } => {
                let locs: Vec<String> = locations
                    .iter()
                    .map(|(p, layer)| format!("{}({})", p.display(), layer))
                    .collect();
                write!(
                    f,
                    "CA: Dual layer definition for {}: [{}]",
                    type_name,
                    locs.join(", ")
                )
            }
            Self::ServerCreatingServices {
                file,
                line,
                service_name,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "CA: Server creating service: {}:{} - {} ({})",
                    file.display(),
                    line,
                    service_name,
                    suggestion
                )
            }
            Self::ApplicationImportsServer {
                file,
                line,
                import_statement,
                ..
            } => {
                write!(
                    f,
                    "CA: Application imports server: {}:{} - {}",
                    file.display(),
                    line,
                    import_statement
                )
            }
        }
    }
}

/// Organization validator
pub struct OrganizationValidator {
    config: ValidationConfig,
}

impl OrganizationValidator {
    /// Create a new organization validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all organization validations
    pub fn validate_all(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_magic_numbers()?);
        violations.extend(self.validate_duplicate_strings()?);
        violations.extend(self.validate_file_placement()?);
        violations.extend(self.validate_trait_placement()?);
        violations.extend(self.validate_declaration_collisions()?);
        violations.extend(self.validate_layer_violations()?);
        Ok(violations)
    }

    /// Check for magic numbers (non-trivial numeric literals)
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

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip constants.rs files (they're allowed to have numbers)
                let file_name = entry.path().file_name().and_then(|n| n.to_str());
                if file_name.is_some_and(|n| n.contains("constant") || n.contains("config")) {
                    continue;
                }

                // Skip test files
                let path_str = entry.path().to_string_lossy();
                if path_str.contains("_test.rs") || path_str.contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
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
                        let num = cap.get(1).map(|m| m.as_str()).unwrap_or("");

                        // Skip allowed numbers
                        if allowed.contains(&num) {
                            continue;
                        }

                        // Skip numbers that are clearly part of a constant reference
                        // e.g., _1024, SIZE_16384
                        if line.contains(&format!("_{}", num))
                            || line.contains(&format!("{}_", num))
                        {
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
                            file: entry.path().to_path_buf(),
                            line: line_num + 1,
                            value: num.to_string(),
                            context: trimmed.to_string(),
                            suggestion: "Consider using a named constant".to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for duplicate string literals that should be constants
    pub fn validate_duplicate_strings(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        let mut string_occurrences: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();

        // Pattern for string literals (15+ chars to reduce noise)
        let string_pattern = Regex::new(r#""([^"\\]{15,})""#).expect("Invalid regex");

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip constants files (they define string constants)
                let file_name = entry.path().file_name().and_then(|n| n.to_str());
                if file_name.is_some_and(|n| n.contains("constant")) {
                    continue;
                }

                // Skip test files
                let path_str = entry.path().to_string_lossy();
                if path_str.contains("_test.rs") || path_str.contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
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
                        let string_val = cap.get(1).map(|m| m.as_str()).unwrap_or("");

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
                            .push((entry.path().to_path_buf(), line_num + 1));
                    }
                }
            }
        }

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

    /// Check for files in wrong architectural locations
    pub fn validate_file_placement(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            let crate_name = crate_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let rel_path = entry.path().strip_prefix(&src_dir).ok();
                let path_str = entry.path().to_string_lossy();
                let file_name = entry
                    .path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Check for adapter implementations in domain crate
                if crate_name.contains("domain") && path_str.contains("/adapters/") {
                    violations.push(OrganizationViolation::FileInWrongLocation {
                        file: entry.path().to_path_buf(),
                        current_location: "domain/adapters".to_string(),
                        expected_location: "infrastructure/adapters".to_string(),
                        reason: "Adapters belong in infrastructure layer".to_string(),
                        severity: Severity::Error,
                    });
                }

                // Check for port definitions in infrastructure
                if crate_name.contains("infrastructure") && path_str.contains("/ports/") {
                    violations.push(OrganizationViolation::FileInWrongLocation {
                        file: entry.path().to_path_buf(),
                        current_location: "infrastructure/ports".to_string(),
                        expected_location: "domain/ports".to_string(),
                        reason: "Ports (interfaces) belong in domain layer".to_string(),
                        severity: Severity::Error,
                    });
                }

                // Check for config files outside config directories
                if file_name.contains("config")
                    && !path_str.contains("/config/")
                    && !path_str.contains("/config.rs")
                {
                    // Allow config.rs at root level
                    if rel_path.is_some_and(|p| p.components().count() > 1) {
                        violations.push(OrganizationViolation::FileInWrongLocation {
                            file: entry.path().to_path_buf(),
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
                            file: entry.path().to_path_buf(),
                            current_location: "nested error.rs".to_string(),
                            expected_location: "crate root or error/ module".to_string(),
                            reason: "Error types should be centralized".to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for traits defined outside domain/ports
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

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            let crate_name = crate_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip domain crate (traits are allowed there)
            if crate_name.contains("domain") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path_str = entry.path().to_string_lossy();

                // Skip if in ports directory (re-exports are OK)
                if path_str.contains("/ports/") {
                    continue;
                }

                // Skip DI modules (they often define internal traits)
                if path_str.contains("/di/") {
                    continue;
                }

                // Skip test files
                if path_str.contains("_test.rs") || path_str.contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
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
                        let trait_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

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
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                trait_name: trait_name.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Check for declaration collisions (same name defined in multiple places)
    pub fn validate_declaration_collisions(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();
        let mut declarations: HashMap<String, Vec<(PathBuf, usize, String)>> = HashMap::new();

        let struct_pattern =
            Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");
        let enum_pattern =
            Regex::new(r"(?:pub\s+)?enum\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");
        let trait_pattern =
            Regex::new(r"(?:pub\s+)?trait\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid regex");

        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Check structs
                    if let Some(cap) = struct_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        declarations.entry(name.to_string()).or_default().push((
                            entry.path().to_path_buf(),
                            line_num + 1,
                            "struct".to_string(),
                        ));
                    }

                    // Check enums
                    if let Some(cap) = enum_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        declarations.entry(name.to_string()).or_default().push((
                            entry.path().to_path_buf(),
                            line_num + 1,
                            "enum".to_string(),
                        ));
                    }

                    // Check traits
                    if let Some(cap) = trait_pattern.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        declarations.entry(name.to_string()).or_default().push((
                            entry.path().to_path_buf(),
                            line_num + 1,
                            "trait".to_string(),
                        ));
                    }
                }
            }
        }

        // Report names with multiple declarations
        for (name, locations) in declarations {
            // Check if declarations are in different crates
            let unique_crates: HashSet<_> = locations
                .iter()
                .filter_map(|(path, _, _)| {
                    path.components()
                        .find(|c| c.as_os_str().to_string_lossy().starts_with("mcb-"))
                })
                .collect();

            if unique_crates.len() > 1 {
                // Skip common names that are expected to have multiple declarations
                let common_names = ["Error", "Result", "Config", "Options", "Builder"];
                if common_names.contains(&name.as_str()) {
                    continue;
                }

                violations.push(OrganizationViolation::DeclarationCollision {
                    name,
                    locations,
                    severity: Severity::Info,
                });
            }
        }

        Ok(violations)
    }

    /// Validate Clean Architecture layer violations
    pub fn validate_layer_violations(&self) -> Result<Vec<OrganizationViolation>> {
        let mut violations = Vec::new();

        // Patterns for detecting layer violations
        let arc_new_service_pattern =
            Regex::new(r"Arc::new\s*\(\s*([A-Z][a-zA-Z0-9_]*(?:Service|Provider|Repository))::new")
                .expect("Invalid regex");
        let server_import_pattern =
            Regex::new(r"use\s+(?:crate::|super::)*server::").expect("Invalid regex");

        for src_dir in self.config.get_scan_dirs()? {
            // Skip mcb-validate itself
            if src_dir.to_string_lossy().contains("mcb-validate") {
                continue;
            }

            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Skip test files
                if path_str.contains("_test.rs") || path_str.contains("/tests/") {
                    continue;
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

                    // Track brace depth
                    brace_depth += line.chars().filter(|c| *c == '{').count() as i32;
                    brace_depth -= line.chars().filter(|c| *c == '}').count() as i32;

                    // Exit test module when braces close (use < not <= to avoid premature exit)
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
                    if is_server_layer {
                        if let Some(cap) = arc_new_service_pattern.captures(line) {
                            let service_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");

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
            }
        }

        Ok(violations)
    }

    fn get_crate_dirs(&self) -> Result<Vec<PathBuf>> {
        self.config.get_source_dirs()
    }

    /// Check if a path is from legacy/additional source directories
    #[allow(dead_code)]
    fn is_legacy_path(&self, path: &std::path::Path) -> bool {
        self.config.is_legacy_path(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_crate(temp: &TempDir, name: &str, content: &str) {
        let crate_dir = temp.path().join("crates").join(name).join("src");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(crate_dir.join("lib.rs"), content).unwrap();

        let cargo_dir = temp.path().join("crates").join(name);
        fs::write(
            cargo_dir.join("Cargo.toml"),
            format!(
                r#"
[package]
name = "{}"
version = "0.1.0"
"#,
                name
            ),
        )
        .unwrap();
    }

    #[test]
    fn test_magic_number_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub fn process_data() {
    let timeout = 300000;  // magic number (5+ digits)
    let buffer_size = 163840;  // magic number (5+ digits)
}
"#,
        );

        let validator = OrganizationValidator::new(temp.path());
        let violations = validator.validate_magic_numbers().unwrap();

        assert!(violations.len() >= 1, "Should detect magic numbers");
    }

    #[test]
    fn test_constants_file_exemption() {
        let temp = TempDir::new().unwrap();

        let crate_dir = temp.path().join("crates").join("mcb-test").join("src");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(
            crate_dir.join("constants.rs"),
            r#"
pub const TIMEOUT_MS: u64 = 300000;
pub const BUFFER_SIZE: usize = 163840;
"#,
        )
        .unwrap();

        fs::write(
            temp.path()
                .join("crates")
                .join("mcb-test")
                .join("Cargo.toml"),
            r#"
[package]
name = "mcb-test"
version = "0.1.0"
"#,
        )
        .unwrap();

        let validator = OrganizationValidator::new(temp.path());
        let violations = validator.validate_magic_numbers().unwrap();

        assert!(violations.is_empty(), "Constants files should be exempt");
    }
}
