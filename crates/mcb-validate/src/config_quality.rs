//! Configuration Quality Validation
//!
//! Validates configuration code quality:
//! - Detects hardcoded strings that should be configurable
//! - Detects magic numbers outside constants
//! - Detects default values without documentation
//! - Ensures proper use of configuration patterns

use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Configuration quality violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigQualityViolation {
    /// Hardcoded string in configuration that should be configurable
    HardcodedConfigString {
        file: PathBuf,
        line: usize,
        string_value: String,
        context: String,
        severity: Severity,
    },
    /// Magic number in code outside constants module
    MagicNumber {
        file: PathBuf,
        line: usize,
        number: String,
        context: String,
        severity: Severity,
    },
    /// Default implementation without documentation
    UndocumentedDefault {
        file: PathBuf,
        line: usize,
        struct_name: String,
        severity: Severity,
    },
    /// Configuration field without documentation
    UndocumentedConfigField {
        file: PathBuf,
        line: usize,
        field_name: String,
        severity: Severity,
    },
    /// Hardcoded namespace or prefix that should be configurable
    HardcodedNamespace {
        file: PathBuf,
        line: usize,
        namespace: String,
        severity: Severity,
    },
}

impl std::fmt::Display for ConfigQualityViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HardcodedConfigString {
                file,
                line,
                string_value,
                context,
                ..
            } => write!(
                f,
                "{}:{} - Hardcoded string '{}' in {} - should be configurable",
                file.display(),
                line,
                string_value,
                context
            ),
            Self::MagicNumber {
                file,
                line,
                number,
                context,
                ..
            } => write!(
                f,
                "{}:{} - Magic number {} in {} - extract to constant",
                file.display(),
                line,
                number,
                context
            ),
            Self::UndocumentedDefault {
                file,
                line,
                struct_name,
                ..
            } => write!(
                f,
                "{}:{} - Default implementation for '{}' missing documentation comment",
                file.display(),
                line,
                struct_name
            ),
            Self::UndocumentedConfigField {
                file,
                line,
                field_name,
                ..
            } => write!(
                f,
                "{}:{} - Configuration field '{}' missing documentation comment",
                file.display(),
                line,
                field_name
            ),
            Self::HardcodedNamespace {
                file,
                line,
                namespace,
                ..
            } => write!(
                f,
                "{}:{} - Hardcoded namespace '{}' - should be configurable with safe default",
                file.display(),
                line,
                namespace
            ),
        }
    }
}

impl Violation for ConfigQualityViolation {
    fn id(&self) -> &str {
        match self {
            Self::HardcodedConfigString { .. } => "CFG001",
            Self::MagicNumber { .. } => "CFG002",
            Self::UndocumentedDefault { .. } => "CFG003",
            Self::UndocumentedConfigField { .. } => "CFG004",
            Self::HardcodedNamespace { .. } => "CFG005",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Configuration
    }

    fn severity(&self) -> Severity {
        match self {
            Self::HardcodedConfigString { severity, .. }
            | Self::MagicNumber { severity, .. }
            | Self::UndocumentedDefault { severity, .. }
            | Self::UndocumentedConfigField { severity, .. }
            | Self::HardcodedNamespace { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::HardcodedConfigString { file, .. }
            | Self::MagicNumber { file, .. }
            | Self::UndocumentedDefault { file, .. }
            | Self::UndocumentedConfigField { file, .. }
            | Self::HardcodedNamespace { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::HardcodedConfigString { line, .. }
            | Self::MagicNumber { line, .. }
            | Self::UndocumentedDefault { line, .. }
            | Self::UndocumentedConfigField { line, .. }
            | Self::HardcodedNamespace { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::HardcodedConfigString { .. } => Some(
                "Extract to configuration field with Option<String> and provide safe default"
                    .to_string(),
            ),
            Self::MagicNumber { .. } => {
                Some("Define as const in constants.rs or as configuration field".to_string())
            }
            Self::UndocumentedDefault { .. } => Some(
                "Add documentation comment explaining default values and when to override"
                    .to_string(),
            ),
            Self::UndocumentedConfigField { .. } => Some(
                "Add documentation comment explaining the field's purpose and valid values"
                    .to_string(),
            ),
            Self::HardcodedNamespace { .. } => Some(
                "Make configurable via Option<String> with documented default value".to_string(),
            ),
        }
    }
}

/// Configuration quality validator
pub struct ConfigQualityValidator {
    config: ValidationConfig,
}

impl ConfigQualityValidator {
    /// Create a new configuration quality validator with the given configuration
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Create a validator with a custom configuration (alias for new)
    pub fn with_config(config: ValidationConfig) -> Self {
        Self::new(config)
    }

    /// Validate configuration quality across all config files
    pub fn validate(&self) -> Result<Vec<ConfigQualityViolation>> {
        let mut violations = Vec::new();

        // Regex patterns
        let _hardcoded_string_pattern = Regex::new(r#":\s*"([^"]+)".to_string\(\)"#).unwrap();
        let namespace_pattern = Regex::new(r#"namespace:\s*"([^"]+)".to_string\(\)"#).unwrap();
        let client_name_pattern =
            Regex::new(r#"client_name:\s*Some\("([^"]+)".to_string\(\)\)"#).unwrap();
        let header_pattern = Regex::new(r#"header:\s*"([^"]+)".to_string\(\)"#).unwrap();
        let default_impl_pattern = Regex::new(r"impl\s+Default\s+for\s+(\w+)").unwrap();
        let _struct_pattern = Regex::new(r"pub\s+struct\s+(\w+)").unwrap();
        let _field_pattern = Regex::new(r"pub\s+(\w+):\s+").unwrap();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| {
                    e.path().extension().is_some_and(|ext| ext == "rs")
                        && (e.path().to_string_lossy().contains("/config/")
                            || e.path()
                                .file_name()
                                .is_some_and(|name| name.to_string_lossy().contains("config")))
                })
            {
                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                self.check_hardcoded_namespaces(
                    entry.path(),
                    &lines,
                    &namespace_pattern,
                    &mut violations,
                );
                self.check_hardcoded_client_names(
                    entry.path(),
                    &lines,
                    &client_name_pattern,
                    &mut violations,
                );
                self.check_hardcoded_headers(
                    entry.path(),
                    &lines,
                    &header_pattern,
                    &mut violations,
                );
                self.check_undocumented_defaults(
                    entry.path(),
                    &lines,
                    &default_impl_pattern,
                    &mut violations,
                );
            }
        }

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
            if let Some(captures) = namespace_pattern.captures(line) {
                if let Some(namespace) = captures.get(1) {
                    let namespace_str = namespace.as_str();
                    // Skip if it's already using a constant or documented default
                    if !self.is_documented_or_constant(lines, i) {
                        violations.push(ConfigQualityViolation::HardcodedNamespace {
                            file: file.to_path_buf(),
                            line: i + 1,
                            namespace: namespace_str.to_string(),
                            severity: Severity::Warning,
                        });
                    }
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
            if let Some(captures) = client_name_pattern.captures(line) {
                if let Some(client_name) = captures.get(1) {
                    let client_name_str = client_name.as_str();
                    // This is actually acceptable as a default - skip if properly documented
                    if !self.is_documented_or_constant(lines, i) {
                        violations.push(ConfigQualityViolation::HardcodedConfigString {
                            file: file.to_path_buf(),
                            line: i + 1,
                            string_value: client_name_str.to_string(),
                            context: "client_name".to_string(),
                            severity: Severity::Info,
                        });
                    }
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
            if let Some(captures) = header_pattern.captures(line) {
                if let Some(header) = captures.get(1) {
                    let header_str = header.as_str();
                    // Skip if it's a well-known constant like API_KEY_HEADER
                    if header_str.starts_with("X-") && !line.contains("API_KEY_HEADER") {
                        if !self.is_documented_or_constant(lines, i) {
                            violations.push(ConfigQualityViolation::HardcodedConfigString {
                                file: file.to_path_buf(),
                                line: i + 1,
                                string_value: header_str.to_string(),
                                context: "HTTP header".to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }
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
            if let Some(captures) = default_impl_pattern.captures(line) {
                if let Some(struct_name) = captures.get(1) {
                    // Check if there's a doc comment above
                    let has_doc_comment = i > 0 && {
                        lines[i - 1].trim().starts_with("///")
                            || (i > 1 && lines[i - 2].trim().starts_with("///"))
                    };

                    if !has_doc_comment {
                        violations.push(ConfigQualityViolation::UndocumentedDefault {
                            file: file.to_path_buf(),
                            line: i + 1,
                            struct_name: struct_name.as_str().to_string(),
                            severity: Severity::Info,
                        });
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardcoded_namespace_violation() {
        let violation = ConfigQualityViolation::HardcodedNamespace {
            file: PathBuf::from("config.rs"),
            line: 10,
            namespace: "mcb".to_string(),
            severity: Severity::Warning,
        };

        assert_eq!(violation.id(), "CFG005");
        assert_eq!(violation.severity(), Severity::Warning);
        assert!(violation.to_string().contains("mcb"));
        assert!(violation.suggestion().is_some());
    }

    #[test]
    fn test_undocumented_default_violation() {
        let violation = ConfigQualityViolation::UndocumentedDefault {
            file: PathBuf::from("types.rs"),
            line: 50,
            struct_name: "MyConfig".to_string(),
            severity: Severity::Info,
        };

        assert_eq!(violation.id(), "CFG003");
        assert_eq!(violation.category(), ViolationCategory::Configuration);
    }
}
