//! Documentation Completeness Validation
//!
//! Validates documentation:
//! - All pub items have rustdoc (///)
//! - Module-level documentation (//!)
//! - Example code blocks for traits

use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::scan::for_each_crate_rs_path;
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig, ValidationError};

/// Documentation violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationViolation {
    /// Missing module-level documentation
    MissingModuleDoc {
        /// File that is missing module-level documentation (`//!`).
        file: PathBuf,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Missing documentation on public item
    MissingPubItemDoc {
        /// File containing the undocumented public item.
        file: PathBuf,
        /// Line number where the public item is defined.
        line: usize,
        /// Name of the public item missing documentation.
        item_name: String,
        /// Kind of item missing documentation (e.g., "struct", "enum", "trait", "function").
        item_kind: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Missing example code in documentation
    MissingExampleCode {
        /// File containing the item missing an example in its documentation.
        file: PathBuf,
        /// Line number where the item is defined.
        line: usize,
        /// Name of the item missing an example section.
        item_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
}

impl DocumentationViolation {
    /// Returns the severity level of this violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}

impl std::fmt::Display for DocumentationViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingModuleDoc { file, .. } => {
                write!(f, "Missing module doc: {}", file.display())
            }
            Self::MissingPubItemDoc {
                file,
                line,
                item_name,
                item_kind,
                ..
            } => {
                write!(
                    f,
                    "Missing {} doc: {}:{} - {}",
                    item_kind,
                    file.display(),
                    line,
                    item_name
                )
            }
            Self::MissingExampleCode {
                file,
                line,
                item_name,
                ..
            } => {
                write!(
                    f,
                    "Missing example: {}:{} - {}",
                    file.display(),
                    line,
                    item_name
                )
            }
        }
    }
}

impl Violation for DocumentationViolation {
    fn id(&self) -> &str {
        match self {
            Self::MissingModuleDoc { .. } => "DOC001",
            Self::MissingPubItemDoc { .. } => "DOC002",
            Self::MissingExampleCode { .. } => "DOC003",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Documentation
    }

    fn severity(&self) -> Severity {
        match self {
            Self::MissingModuleDoc { severity, .. }
            | Self::MissingPubItemDoc { severity, .. }
            | Self::MissingExampleCode { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::MissingModuleDoc { file, .. }
            | Self::MissingPubItemDoc { file, .. }
            | Self::MissingExampleCode { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::MissingModuleDoc { .. } => None,
            Self::MissingPubItemDoc { line, .. } | Self::MissingExampleCode { line, .. } => {
                Some(*line)
            }
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::MissingModuleDoc { .. } => {
                Some("Add //! module-level documentation at the top of the file".to_string())
            }
            Self::MissingPubItemDoc {
                item_kind,
                item_name,
                ..
            } => Some(format!("Add /// documentation for {item_kind} {item_name}")),
            Self::MissingExampleCode { item_name, .. } => Some(format!(
                "Add # Example section to {item_name} documentation"
            )),
        }
    }
}

/// Validates documentation completeness for public items in Rust crates.
///
/// Ensures all public items (structs, enums, traits, functions) have rustdoc comments (///)
/// and that module-level documentation exists. Optionally checks for example code in trait documentation.
pub struct DocumentationValidator {
    config: ValidationConfig,
}

impl DocumentationValidator {
    /// Create a new documentation validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration for multi-directory support
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all documentation validations
    pub fn validate_all(&self) -> Result<Vec<DocumentationViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_module_docs()?);
        violations.extend(self.validate_pub_item_docs()?);
        Ok(violations)
    }

    /// Verify module-level documentation exists
    pub fn validate_module_docs(&self) -> Result<Vec<DocumentationViolation>> {
        let mut violations = Vec::new();
        let module_doc_pattern = Regex::new(r"^//!")
            .map_err(|e| ValidationError::InvalidRegex(format!("module doc pattern: {e}")))?;

        for_each_crate_rs_path(&self.config, |path, _src_dir, _crate_name| {
            let content = std::fs::read_to_string(path)?;
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            // Only check lib.rs, mod.rs, and main module files
            if file_name != "lib.rs" && file_name != "mod.rs" {
                return Ok(());
            }

            // Check if first non-empty line is module doc
            let has_module_doc = content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .take(1)
                .any(|line| module_doc_pattern.is_match(line));

            if !has_module_doc {
                violations.push(DocumentationViolation::MissingModuleDoc {
                    file: path.to_path_buf(),
                    severity: Severity::Warning,
                });
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Verify all pub items have rustdoc
    pub fn validate_pub_item_docs(&self) -> Result<Vec<DocumentationViolation>> {
        let mut violations = Vec::new();

        // Patterns for public items
        let pub_struct_pattern = Regex::new(r"pub\s+struct\s+([A-Z][a-zA-Z0-9_]*)")
            .map_err(|e| ValidationError::InvalidRegex(format!("pub struct pattern: {e}")))?;
        let pub_enum_pattern = Regex::new(r"pub\s+enum\s+([A-Z][a-zA-Z0-9_]*)")
            .map_err(|e| ValidationError::InvalidRegex(format!("pub enum pattern: {e}")))?;
        let pub_trait_pattern = Regex::new(r"pub\s+trait\s+([A-Z][a-zA-Z0-9_]*)")
            .map_err(|e| ValidationError::InvalidRegex(format!("pub trait pattern: {e}")))?;
        let pub_fn_pattern = Regex::new(r"pub\s+(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)")
            .map_err(|e| ValidationError::InvalidRegex(format!("pub fn pattern: {e}")))?;
        let _doc_comment_pattern = Regex::new(r"^\s*///")
            .map_err(|e| ValidationError::InvalidRegex(format!("doc comment pattern: {e}")))?;
        let example_pattern = Regex::new(r"#\s*Example").unwrap();

        for_each_crate_rs_path(&self.config, |path, _src_dir, _crate_name| {
            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();

            for (line_num, line) in lines.iter().enumerate() {
                // Check for public structs
                if let Some(cap) = pub_struct_pattern.captures(line) {
                    let name = cap.get(1).map_or("", |m: regex::Match| m.as_str());
                    if !self.has_doc_comment(&lines, line_num) {
                        violations.push(DocumentationViolation::MissingPubItemDoc {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            item_name: name.to_string(),
                            item_kind: "struct".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Check for public enums
                if let Some(cap) = pub_enum_pattern.captures(line) {
                    let name = cap.get(1).map_or("", |m: regex::Match| m.as_str());
                    if !self.has_doc_comment(&lines, line_num) {
                        violations.push(DocumentationViolation::MissingPubItemDoc {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            item_name: name.to_string(),
                            item_kind: "enum".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Check for public traits
                if let Some(cap) = pub_trait_pattern.captures(line) {
                    let name = cap.get(1).map_or("", |m: regex::Match| m.as_str());
                    let path_str = path.to_string_lossy();

                    if self.has_doc_comment(&lines, line_num) {
                        // Check for example code in trait documentation
                        // Skip DI module traits and port traits - they are interface definitions
                        // that don't need examples (they define contracts for DI injection)
                        let is_di_or_port_trait =
                            path_str.contains("/di/modules/") || path_str.contains("/ports/");

                        if !is_di_or_port_trait {
                            let doc_section = self.get_doc_comment_section(&lines, line_num);
                            if !example_pattern.is_match(&doc_section) {
                                violations.push(DocumentationViolation::MissingExampleCode {
                                    file: path.to_path_buf(),
                                    line: line_num + 1,
                                    item_name: name.to_string(),
                                    severity: Severity::Info,
                                });
                            }
                        }
                    } else {
                        violations.push(DocumentationViolation::MissingPubItemDoc {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            item_name: name.to_string(),
                            item_kind: "trait".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }

                // Check for public functions (only top-level, not in impl blocks)
                if let Some(cap) = pub_fn_pattern.captures(line) {
                    let name = cap.get(1).map_or("", |m: regex::Match| m.as_str());

                    // Skip methods in impl blocks (approximation: indentation > 0)
                    if line.starts_with("    ") || line.starts_with('\t') {
                        continue;
                    }

                    if !self.has_doc_comment(&lines, line_num) {
                        violations.push(DocumentationViolation::MissingPubItemDoc {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            item_name: name.to_string(),
                            item_kind: "function".to_string(),
                            severity: Severity::Info,
                        });
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Checks if a line has a documentation comment above it.
    ///
    /// Looks backwards from the item line to find a `///` doc comment,
    /// skipping attributes and empty lines.
    fn has_doc_comment(&self, lines: &[&str], item_line: usize) -> bool {
        let doc_pattern = Regex::new(r"^\s*///").unwrap();
        let attr_pattern = Regex::new(r"^\s*#\[").unwrap();

        if item_line == 0 {
            return false;
        }

        // Look backwards for doc comments, skipping attributes
        let mut i = item_line - 1;
        loop {
            let line = lines[i].trim();

            // Skip empty lines between attributes and doc comments
            if line.is_empty() {
                if i == 0 {
                    return false;
                }
                i -= 1;
                continue;
            }

            // Skip attributes
            if attr_pattern.is_match(lines[i]) {
                if i == 0 {
                    return false;
                }
                i -= 1;
                continue;
            }

            // Check for doc comment
            return doc_pattern.is_match(lines[i]);
        }
    }

    /// Extracts the complete documentation comment section for an item.
    ///
    /// Collects all consecutive `///` lines above the item, skipping attributes,
    /// and returns them as a single string for analysis.
    fn get_doc_comment_section(&self, lines: &[&str], item_line: usize) -> String {
        let doc_pattern = Regex::new(r"^\s*///(.*)").unwrap();
        let attr_pattern = Regex::new(r"^\s*#\[").unwrap();

        if item_line == 0 {
            return String::new();
        }

        let mut doc_lines = Vec::new();
        let mut i = item_line - 1;

        loop {
            let line = lines[i];

            // Skip attributes
            if attr_pattern.is_match(line) {
                if i == 0 {
                    break;
                }
                i -= 1;
                continue;
            }

            // Collect doc comment
            if let Some(cap) = doc_pattern.captures(line) {
                let content = cap.get(1).map_or("", |m| m.as_str());
                doc_lines.push(content);
            } else if !line.trim().is_empty() {
                break;
            }

            if i == 0 {
                break;
            }
            i -= 1;
        }

        doc_lines.reverse();
        doc_lines.join("\n")
    }
}

impl_validator!(
    DocumentationValidator,
    "documentation",
    "Validates documentation standards"
);
