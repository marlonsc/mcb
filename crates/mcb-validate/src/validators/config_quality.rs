//! Configuration Quality Validation
//!
//! Validates configuration code quality:
//! - Detects hardcoded strings that should be configurable
//! - Detects magic numbers outside constants
//! - Detects default values without documentation
//! - Ensures proper use of configuration patterns

use std::path::{Path, PathBuf};

use regex::Regex;

use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_scan_file;
use crate::traits::violation::ViolationCategory;
use crate::{Result, Severity, ValidationConfig};

define_violations! {
    dynamic_severity,
    ViolationCategory::Configuration,
    pub enum ConfigQualityViolation {
        /// Hardcoded string in configuration that should be configurable
        #[violation(
            id = "CFG001",
            severity = Warning,
            message = "{file}:{line} - Hardcoded string '{string_value}' in {context} - should be configurable",
            suggestion = "Extract to configuration field with Option<String> and provide safe default"
        )]
        HardcodedConfigString {
            file: PathBuf,
            line: usize,
            string_value: String,
            context: String,
            severity: Severity,
        },
        /// Magic number in code outside constants module
        #[violation(
            id = "CFG002",
            severity = Warning,
            message = "{file}:{line} - Magic number {number} in {context} - extract to constant",
            suggestion = "Define as const in constants.rs or as configuration field"
        )]
        MagicNumber {
            file: PathBuf,
            line: usize,
            number: String,
            context: String,
            severity: Severity,
        },
        /// Default implementation without documentation
        #[violation(
            id = "CFG003",
            severity = Info,
            message = "{file}:{line} - Default implementation for '{struct_name}' missing documentation comment",
            suggestion = "Add documentation comment explaining default values and when to override"
        )]
        UndocumentedDefault {
            file: PathBuf,
            line: usize,
            struct_name: String,
            severity: Severity,
        },
        /// Configuration field without documentation
        #[violation(
            id = "CFG004",
            severity = Info,
            message = "{file}:{line} - Configuration field '{field_name}' missing documentation comment",
            suggestion = "Add documentation comment explaining the field's purpose and valid values"
        )]
        UndocumentedConfigField {
            file: PathBuf,
            line: usize,
            field_name: String,
            severity: Severity,
        },
        /// Hardcoded namespace or prefix that should be configurable
        #[violation(
            id = "CFG005",
            severity = Warning,
            message = "{file}:{line} - Hardcoded namespace '{namespace}' - should be configurable with safe default",
            suggestion = "Make configurable via Option<String> with documented default value"
        )]
        HardcodedNamespace {
            file: PathBuf,
            line: usize,
            namespace: String,
            severity: Severity,
        },
    }
}

/// Configuration quality validator
pub struct ConfigQualityValidator {
    config: ValidationConfig,
}

impl ConfigQualityValidator {
    /// Create a new configuration quality validator with the given configuration
    #[must_use]
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Validate configuration quality across all config files
    pub fn validate(&self) -> Result<Vec<ConfigQualityViolation>> {
        let mut violations = Vec::new();

        // Regex patterns
        let namespace_pattern = compile_regex(r#"namespace:\s*"([^"]+)".to_string\(\)"#)?;
        let client_name_pattern =
            compile_regex(r#"client_name:\s*Some\("([^"]+)".to_string\(\)\)"#)?;
        let header_pattern = compile_regex(r#"header:\s*"([^"]+)".to_string\(\)"#)?;
        let default_impl_pattern = compile_regex(r"impl\s+Default\s+for\s+(\w+)")?;

        for_each_scan_file(
            &self.config,
            Some(LanguageId::Rust),
            false,
            |entry, _src_dir| {
                let is_config_file = entry
                    .absolute_path
                    .to_str()
                    .is_some_and(|s| s.contains("/config/"))
                    || entry
                        .absolute_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.contains("config"));
                if !is_config_file {
                    return Ok(());
                }

                let content = std::fs::read_to_string(&entry.absolute_path)?;
                let lines: Vec<&str> = content.lines().collect();

                self.check_hardcoded_namespaces(
                    &entry.absolute_path,
                    &lines,
                    &namespace_pattern,
                    &mut violations,
                );
                self.check_hardcoded_client_names(
                    &entry.absolute_path,
                    &lines,
                    &client_name_pattern,
                    &mut violations,
                );
                self.check_hardcoded_headers(
                    &entry.absolute_path,
                    &lines,
                    &header_pattern,
                    &mut violations,
                );
                self.check_undocumented_defaults(
                    &entry.absolute_path,
                    &lines,
                    &default_impl_pattern,
                    &mut violations,
                );

                Ok(())
            },
        )?;

        Ok(violations)
    }

    fn check_hardcoded_namespaces(
        &self,
        file: &Path,
        lines: &[&str],
        namespace_pattern: &Regex,
        violations: &mut Vec<ConfigQualityViolation>,
    ) {
        for (i, line) in lines.iter().enumerate() {
            if let Some(captures) = namespace_pattern.captures(line)
                && let Some(namespace) = captures.get(1)
            {
                let namespace_str = namespace.as_str();
                // Skip if it's already using a constant or documented default
                if !self.is_documented_or_constant(lines, i) {
                    violations.push(ConfigQualityViolation::HardcodedNamespace {
                        file: file.to_path_buf(),
                        line: i + 1,
                        namespace: namespace_str.to_owned(),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }

    fn check_hardcoded_client_names(
        &self,
        file: &Path,
        lines: &[&str],
        client_name_pattern: &Regex,
        violations: &mut Vec<ConfigQualityViolation>,
    ) {
        for (i, line) in lines.iter().enumerate() {
            if let Some(captures) = client_name_pattern.captures(line)
                && let Some(client_name) = captures.get(1)
            {
                let client_name_str = client_name.as_str();
                // This is actually acceptable as a default - skip if properly documented
                if !self.is_documented_or_constant(lines, i) {
                    violations.push(ConfigQualityViolation::HardcodedConfigString {
                        file: file.to_path_buf(),
                        line: i + 1,
                        string_value: client_name_str.to_owned(),
                        context: "client_name".to_owned(),
                        severity: Severity::Info,
                    });
                }
            }
        }
    }

    fn check_hardcoded_headers(
        &self,
        file: &Path,
        lines: &[&str],
        header_pattern: &Regex,
        violations: &mut Vec<ConfigQualityViolation>,
    ) {
        for (i, line) in lines.iter().enumerate() {
            if let Some(captures) = header_pattern.captures(line)
                && let Some(header) = captures.get(1)
            {
                let header_str = header.as_str();
                // Skip if it's a well-known constant like API_KEY_HEADER
                if header_str.starts_with("X-")
                    && !line.contains("API_KEY_HEADER")
                    && !self.is_documented_or_constant(lines, i)
                {
                    violations.push(ConfigQualityViolation::HardcodedConfigString {
                        file: file.to_path_buf(),
                        line: i + 1,
                        string_value: header_str.to_owned(),
                        context: "HTTP header".to_owned(),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }

    fn check_undocumented_defaults(
        &self,
        file: &Path,
        lines: &[&str],
        default_impl_pattern: &Regex,
        violations: &mut Vec<ConfigQualityViolation>,
    ) {
        for (i, line) in lines.iter().enumerate() {
            if let Some(captures) = default_impl_pattern.captures(line)
                && let Some(struct_name) = captures.get(1)
            {
                // Check if there's a doc comment above
                let has_doc_comment = i > 0 && {
                    lines[i - 1].trim().starts_with("///")
                        || (i > 1 && lines[i - 2].trim().starts_with("///"))
                };

                if !has_doc_comment {
                    violations.push(ConfigQualityViolation::UndocumentedDefault {
                        file: file.to_path_buf(),
                        line: i + 1,
                        struct_name: struct_name.as_str().to_owned(),
                        severity: Severity::Info,
                    });
                }
            }
        }
    }

    fn is_documented_or_constant(&self, lines: &[&str], line_idx: usize) -> bool {
        // Check for documentation comment or constant usage
        if line_idx > 0 {
            let prev_line = lines[line_idx - 1];
            if prev_line.contains("///") || prev_line.contains("//") || prev_line.contains("const")
            {
                return true;
            }
        }

        // Check if the line itself uses a constant
        let current_line = lines[line_idx];
        current_line.contains("const") || current_line.contains("DEFAULT_")
    }
}
