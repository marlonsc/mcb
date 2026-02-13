//! Naming Convention Validation
//!
//! Validates naming conventions:
//! - Structs/Enums/Traits: CamelCase
//! - Functions/Methods: `snake_case`
//! - Constants: `SCREAMING_SNAKE_CASE`
//! - Modules/Files: `snake_case`

use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::config::NamingRulesConfig;
use crate::run_context::ValidationRunContext;
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};

define_violations! {
    ViolationCategory::Naming,
    pub enum NamingViolation {
        /// Bad struct/enum/trait name (should be CamelCase)
        violation(
            id = "NAME001",
            severity = Warning,
            message = "Bad type name: {file}:{line} - {name} (expected {expected_case})",
            suggestion = "Rename '{name}' to {expected_case} format"
        )
        BadTypeName {
            file: PathBuf,
            line: usize,
            name: String,
            expected_case: String,
            severity: Severity,
        },
        /// Bad function/method name (should be snake_case)
        violation(
            id = "NAME002",
            severity = Warning,
            message = "Bad function name: {file}:{line} - {name} (expected {expected_case})",
            suggestion = "Rename '{name}' to {expected_case} format"
        )
        BadFunctionName {
            file: PathBuf,
            line: usize,
            name: String,
            expected_case: String,
            severity: Severity,
        },
        /// Bad constant name (should be SCREAMING_SNAKE_CASE)
        violation(
            id = "NAME003",
            severity = Warning,
            message = "Bad constant name: {file}:{line} - {name} (expected {expected_case})",
            suggestion = "Rename '{name}' to {expected_case} format"
        )
        BadConstantName {
            file: PathBuf,
            line: usize,
            name: String,
            expected_case: String,
            severity: Severity,
        },
        /// Bad module/file name (should be snake_case)
        violation(
            id = "NAME004",
            severity = Warning,
            message = "Bad module name: {path} (expected {expected_case})",
            suggestion = "Rename file/module to {expected_case} format"
        )
        BadModuleName {
            path: PathBuf,
            expected_case: String,
            severity: Severity,
        },
        /// File suffix doesn't match component type
        violation(
            id = "NAME005",
            severity = Warning,
            message = "Bad file suffix: {path} ({component_type}) has suffix '{current_suffix}' but expected '{expected_suffix}'",
            suggestion = "Add '{expected_suffix}' suffix to file name"
        )
        BadFileSuffix {
            path: PathBuf,
            component_type: String,
            current_suffix: String,
            expected_suffix: String,
            severity: Severity,
        },
        /// File name doesn't follow CA naming convention
        violation(
            id = "NAME006",
            severity = Warning,
            message = "CA naming: {path} ({detected_type}): {issue} - {suggestion}"
        )
        BadCaNaming {
            path: PathBuf,
            detected_type: String,
            issue: String,
            suggestion: String,
            severity: Severity,
        },
    }
}

/// Validates naming conventions across Rust code.
///
/// Checks that structs, enums, traits use CamelCase; functions and methods use snake_case;
/// constants use SCREAMING_SNAKE_CASE; and modules/files use snake_case.
pub struct NamingValidator {
    config: ValidationConfig,
    rules: NamingRulesConfig,
}

impl NamingValidator {
    /// Creates a new naming validator, loading configuration from files.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root: PathBuf = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(ValidationConfig::new(root), &file_config.rules.naming)
    }

    /// Creates a validator with custom configuration.
    pub fn with_config(config: ValidationConfig, rules: &NamingRulesConfig) -> Self {
        Self {
            config,
            rules: rules.clone(),
        }
    }

    /// Runs all naming validations and returns collected violations.
    pub fn validate_all(&self) -> Result<Vec<NamingViolation>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }
        let mut violations = Vec::new();
        violations.extend(self.validate_type_names()?);
        violations.extend(self.validate_function_names()?);
        violations.extend(self.validate_constant_names()?);
        violations.extend(self.validate_module_names()?);
        violations.extend(self.validate_file_suffixes()?);
        violations.extend(self.validate_ca_naming()?);
        Ok(violations)
    }

    /// Validates that struct, enum, and trait names follow CamelCase convention.
    pub fn validate_type_names(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        let struct_pattern = Regex::new(r"(?:pub\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)")?;
        let enum_pattern = Regex::new(r"(?:pub\s+)?enum\s+([A-Za-z_][A-Za-z0-9_]*)")?;
        let trait_pattern = Regex::new(r"(?:pub\s+)?trait\s+([A-Za-z_][A-Za-z0-9_]*)")?;

        self.for_each_crate_src_rs_path(|path| {
            let content = std::fs::read_to_string(path)?;

            for (line_num, line) in content.lines().enumerate() {
                // Skip doc comments and regular comments
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }

                // Check structs
                if let Some(cap) = struct_pattern.captures(line) {
                    let name = cap.get(1).map_or("", |m| m.as_str());
                    if !self.is_camel_case(name) {
                        violations.push(NamingViolation::BadTypeName {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            name: name.to_string(),
                            expected_case: "CamelCase".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Check enums
                if let Some(cap) = enum_pattern.captures(line) {
                    let name = cap.get(1).map_or("", |m| m.as_str());
                    if !self.is_camel_case(name) {
                        violations.push(NamingViolation::BadTypeName {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            name: name.to_string(),
                            expected_case: "CamelCase".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Check traits
                if let Some(cap) = trait_pattern.captures(line) {
                    let name = cap.get(1).map_or("", |m| m.as_str());
                    if !self.is_camel_case(name) {
                        violations.push(NamingViolation::BadTypeName {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            name: name.to_string(),
                            expected_case: "CamelCase".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// Validates that function and method names follow snake_case convention.
    pub fn validate_function_names(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        let fn_pattern =
            Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]")?;

        self.for_each_crate_src_rs_path(|path| {
            let content = std::fs::read_to_string(path)?;

            for (line_num, line) in content.lines().enumerate() {
                if let Some(cap) = fn_pattern.captures(line) {
                    let name = cap.get(1).map_or("", |m| m.as_str());

                    // Skip test functions
                    if name.starts_with("test_") {
                        continue;
                    }

                    if !self.is_snake_case(name) {
                        violations.push(NamingViolation::BadFunctionName {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            name: name.to_string(),
                            expected_case: "snake_case".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// Validates that constants and statics follow SCREAMING_SNAKE_CASE convention.
    pub fn validate_constant_names(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        let const_pattern = Regex::new(r"(?:pub\s+)?const\s+([A-Za-z_][A-Za-z0-9_]*)\s*:")?;
        let static_pattern = Regex::new(r"(?:pub\s+)?static\s+([A-Za-z_][A-Za-z0-9_]*)\s*:")?;

        self.for_each_crate_src_rs_path(|path| {
            let content = std::fs::read_to_string(path)?;

            for (line_num, line) in content.lines().enumerate() {
                // Check const
                if let Some(cap) = const_pattern.captures(line) {
                    let name = cap.get(1).map_or("", |m| m.as_str());
                    if !self.is_screaming_snake_case(name) {
                        violations.push(NamingViolation::BadConstantName {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            name: name.to_string(),
                            expected_case: "SCREAMING_SNAKE_CASE".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Check static
                if let Some(cap) = static_pattern.captures(line) {
                    let name = cap.get(1).map_or("", |m| m.as_str());
                    if !self.is_screaming_snake_case(name) {
                        violations.push(NamingViolation::BadConstantName {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            name: name.to_string(),
                            expected_case: "SCREAMING_SNAKE_CASE".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// Validates that module and file names follow snake_case convention.
    pub fn validate_module_names(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        self.for_each_crate_src_rs_path(|path| {
            let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

            // Skip lib.rs, mod.rs, main.rs
            if file_name == "lib" || file_name == "mod" || file_name == "main" {
                return Ok(());
            }

            if !self.is_snake_case(file_name) {
                violations.push(NamingViolation::BadModuleName {
                    path: path.to_path_buf(),
                    expected_case: "snake_case".to_string(),
                    severity: Severity::Warning,
                });
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// Validates that file suffixes match component types per Clean Architecture naming conventions.
    pub fn validate_file_suffixes(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        self.for_each_crate_src_rs_path(|path| {
            let crate_name = self.crate_name_from_path(path);
            let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

            // Skip standard files
            if file_name == "lib" || file_name == "mod" || file_name == "main" {
                return Ok(());
            }

            let path_str = path.to_string_lossy();

            // Check repository files should have _repository suffix
            if (path_str.contains("/repositories/") || path_str.contains("/adapters/repository/"))
                && !file_name.ends_with("_repository")
                && file_name != "mod"
            {
                violations.push(NamingViolation::BadFileSuffix {
                    path: path.to_path_buf(),
                    component_type: "Repository".to_string(),
                    current_suffix: self.get_suffix(file_name).to_string(),
                    expected_suffix: "_repository".to_string(),
                    severity: Severity::Warning,
                });
            }

            // Check handler files in server crate
            if crate_name == self.rules.server_crate && path_str.contains("/handlers/") {
                // Handlers should have descriptive names (snake_case tool names)
                // but NOT have _handler suffix (that's redundant with directory)
                if file_name.ends_with("_handler") {
                    violations.push(NamingViolation::BadFileSuffix {
                        path: path.to_path_buf(),
                        component_type: "Handler".to_string(),
                        current_suffix: "_handler".to_string(),
                        expected_suffix: "<tool_name> (no _handler suffix in handlers/ dir)"
                            .to_string(),
                        severity: Severity::Info,
                    });
                }
            }

            // Check service files should have _service suffix if in services directory
            // Note: mcb-domain/domain_services contains interfaces, not implementations
            // so we skip suffix validation for that directory
            if path_str.contains("/services/")
                && !path_str.contains("/domain_services/")
                && crate_name != self.rules.domain_crate
                && !file_name.ends_with("_service")
                && file_name != "mod"
            {
                violations.push(NamingViolation::BadFileSuffix {
                    path: path.to_path_buf(),
                    component_type: "Service".to_string(),
                    current_suffix: self.get_suffix(file_name).to_string(),
                    expected_suffix: "_service".to_string(),
                    severity: Severity::Info,
                });
            }

            // Check factory files - allow both 'factory.rs' and '*_factory.rs'
            // A file named exactly "factory.rs" is valid (e.g., provider_factory module)
            if file_name.contains("factory")
                && !file_name.ends_with("_factory")
                && file_name != "factory"
            {
                violations.push(NamingViolation::BadFileSuffix {
                    path: path.to_path_buf(),
                    component_type: "Factory".to_string(),
                    current_suffix: self.get_suffix(file_name).to_string(),
                    expected_suffix: "_factory".to_string(),
                    severity: Severity::Info,
                });
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// Validates Clean Architecture naming conventions for files and components.
    pub fn validate_ca_naming(&self) -> Result<Vec<NamingViolation>> {
        let mut violations = Vec::new();

        self.for_each_crate_src_rs_path(|path| {
            let crate_name = self.crate_name_from_path(path);
            let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let path_str = path.to_string_lossy();

            // Skip standard files
            if file_name == "lib" || file_name == "mod" || file_name == "main" {
                return Ok(());
            }

            // Domain crate: port traits should be in ports/
            if crate_name == self.rules.domain_crate {
                // Files with "provider" in name should be in ports/providers/
                if file_name.contains("provider")
                    && !path_str.contains("/ports/providers/")
                    && !path_str.contains("/ports/")
                {
                    violations.push(NamingViolation::BadCaNaming {
                        path: path.to_path_buf(),
                        detected_type: "Provider Port".to_string(),
                        issue: "Provider file outside ports/ directory".to_string(),
                        suggestion: "Move to ports/providers/".to_string(),
                        severity: Severity::Warning,
                    });
                }

                // Files with "repository" in name should be in repositories/
                if file_name.contains("repository") && !path_str.contains("/repositories/") {
                    violations.push(NamingViolation::BadCaNaming {
                        path: path.to_path_buf(),
                        detected_type: "Repository Port".to_string(),
                        issue: "Repository file outside repositories/ directory".to_string(),
                        suggestion: "Move to repositories/".to_string(),
                        severity: Severity::Warning,
                    });
                }
            }

            // Infrastructure crate: adapters should be in adapters/
            if crate_name == self.rules.infrastructure_crate {
                // Implementation files should be in adapters/
                if (file_name.ends_with("_impl") || file_name.contains("adapter"))
                    && !path_str.contains("/adapters/")
                {
                    violations.push(NamingViolation::BadCaNaming {
                        path: path.to_path_buf(),
                        detected_type: "Adapter".to_string(),
                        issue: "Adapter/implementation file outside adapters/ directory"
                            .to_string(),
                        suggestion: "Move to adapters/".to_string(),
                        severity: Severity::Warning,
                    });
                }

                // DI modules should be in di/
                if file_name.contains("module") && !path_str.contains("/di/") {
                    violations.push(NamingViolation::BadCaNaming {
                        path: path.to_path_buf(),
                        detected_type: "DI Module".to_string(),
                        issue: "Module file outside di/ directory".to_string(),
                        suggestion: "Move to di/modules/".to_string(),
                        severity: Severity::Info,
                    });
                }
            }

            // Server crate: handlers should be in handlers/ or admin/
            if crate_name == self.rules.server_crate {
                // Allow handlers in handlers/, admin/, or tools/ directories
                let in_allowed_handler_dir = path_str.contains("/handlers/")
                    || path_str.contains("/admin/")
                    || path_str.contains("/tools/");
                if file_name.contains("handler") && !in_allowed_handler_dir {
                    violations.push(NamingViolation::BadCaNaming {
                        path: path.to_path_buf(),
                        detected_type: "Handler".to_string(),
                        issue: "Handler file outside handlers/ directory".to_string(),
                        suggestion: "Move to handlers/, admin/, or tools/".to_string(),
                        severity: Severity::Warning,
                    });
                }
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// Extracts the suffix from a file name (part after the last underscore).
    fn get_suffix<'a>(&self, name: &'a str) -> &'a str {
        if let Some(pos) = name.rfind('_') {
            &name[pos..]
        } else {
            ""
        }
    }

    /// Checks if a name follows CamelCase convention.
    fn is_camel_case(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Must start with uppercase
        let Some(first_char) = name.chars().next() else {
            return false;
        };
        if !first_char.is_ascii_uppercase() {
            return false;
        }

        // No underscores allowed (except at the start for private items, which we skip)
        if name.contains('_') {
            return false;
        }

        // Must have at least one lowercase letter
        name.chars().any(|c| c.is_ascii_lowercase())
    }

    /// Helper to validate snake-like case conventions.
    fn is_valid_snake_case(&self, name: &str, is_uppercase: bool) -> bool {
        if name.is_empty() {
            return false;
        }

        // Must be all lowercase/uppercase (depending on is_uppercase) or underscores or digits
        for c in name.chars() {
            let valid_case = if is_uppercase {
                c.is_ascii_uppercase()
            } else {
                c.is_ascii_lowercase()
            };
            if !valid_case && c != '_' && !c.is_ascii_digit() {
                return false;
            }
        }

        // Can't start with digit
        !name.chars().next().is_some_and(|c| c.is_ascii_digit())
    }

    /// Checks if a name follows snake_case convention.
    fn is_snake_case(&self, name: &str) -> bool {
        self.is_valid_snake_case(name, false)
    }

    /// Checks if a name follows SCREAMING_SNAKE_CASE convention.
    fn is_screaming_snake_case(&self, name: &str) -> bool {
        self.is_valid_snake_case(name, true)
    }

    fn get_crate_dirs(&self) -> Result<Vec<PathBuf>> {
        self.config.get_source_dirs()
    }

    fn crate_name_from_path(&self, path: &Path) -> String {
        for component in path.components() {
            let component = component.as_os_str().to_string_lossy();
            if component.starts_with("mcb-") {
                return match component.as_ref() {
                    "mcb-domain" => self.rules.domain_crate.clone(),
                    "mcb-infrastructure" => self.rules.infrastructure_crate.clone(),
                    "mcb-server" => self.rules.server_crate.clone(),
                    _ => String::new(),
                };
            }
        }
        String::new()
    }

    fn should_skip_crate(&self, src_dir: &Path) -> bool {
        let path_str = src_dir.to_string_lossy();
        let file_config = crate::config::FileConfig::load(&self.config.workspace_root);
        file_config
            .general
            .skip_crates
            .iter()
            .any(|skip| path_str.contains(skip))
    }

    fn for_each_crate_src_rs_path<F>(&self, mut f: F) -> Result<()>
    where
        F: FnMut(&Path) -> Result<()>,
    {
        let context = ValidationRunContext::active_or_build(&self.config)?;
        for crate_dir in self.get_crate_dirs()? {
            let src_dir = crate_dir.join("src");
            if !src_dir.exists() || self.should_skip_crate(&src_dir) {
                continue;
            }

            for entry in context.file_inventory() {
                if entry.absolute_path.starts_with(&src_dir)
                    && entry
                        .absolute_path
                        .extension()
                        .is_some_and(|ext| ext == "rs")
                {
                    f(&entry.absolute_path)?;
                }
            }
        }
        Ok(())
    }
}

impl_validator!(
    NamingValidator,
    "naming",
    "Validates naming conventions (CamelCase, snake_case, SCREAMING_SNAKE_CASE)"
);
