//! Documentation Completeness Validation
//!
//! Validates documentation:
//! - All pub items have rustdoc (///)
//! - Module-level documentation (//!)
//! - Example code blocks for traits

use crate::filters::LanguageId;
use std::path::PathBuf;
use std::sync::OnceLock;

use regex::Regex;

use crate::pattern_registry::compile_regex;
use crate::scan::for_each_crate_file;
use crate::traits::violation::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};

static DOC_COMMENT_PATTERN: OnceLock<Regex> = OnceLock::new();
static DOC_COMMENT_CAPTURE_PATTERN: OnceLock<Regex> = OnceLock::new();
static ATTR_PATTERN: OnceLock<Regex> = OnceLock::new();

fn doc_comment_pattern() -> &'static Regex {
    DOC_COMMENT_PATTERN.get_or_init(|| compile_regex(r"^\s*///").expect("valid regex literal"))
}

fn doc_comment_capture_pattern() -> &'static Regex {
    DOC_COMMENT_CAPTURE_PATTERN
        .get_or_init(|| compile_regex(r"^\s*///(.*)").expect("valid regex literal"))
}

fn attr_pattern() -> &'static Regex {
    ATTR_PATTERN.get_or_init(|| compile_regex(r"^\s*#\[").expect("valid regex literal"))
}

define_violations! {
    dynamic_severity,
    ViolationCategory::Documentation,
    pub enum DocumentationViolation {
        /// Missing module-level documentation
        #[violation(
            id = "DOC001",
            severity = Warning,
            message = "Missing module doc: {file}",
            suggestion = "Add //! module-level documentation at the top of the file"
        )]
        MissingModuleDoc {
            file: PathBuf,
            severity: Severity,
        },
        /// Missing documentation on public item
        #[violation(
            id = "DOC002",
            severity = Warning,
            message = "Missing {item_kind} doc: {file}:{line} - {item_name}",
            suggestion = "Add /// documentation for {item_kind} {item_name}"
        )]
        MissingPubItemDoc {
            file: PathBuf,
            line: usize,
            item_name: String,
            item_kind: String,
            severity: Severity,
        },
        /// Missing example code in documentation
        #[violation(
            id = "DOC003",
            severity = Info,
            message = "Missing example: {file}:{line} - {item_name}",
            suggestion = "Add # Example section to {item_name} documentation"
        )]
        MissingExampleCode {
            file: PathBuf,
            line: usize,
            item_name: String,
            severity: Severity,
        },
    }
}

impl DocumentationViolation {
    /// Returns the severity level of this violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
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
        let module_doc_pattern = compile_regex(r"^//!")?;

        for_each_crate_file(
            &self.config,
            Some(LanguageId::Rust),
            |entry, _src_dir, _crate_name| {
                let path = &entry.absolute_path;
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
            },
        )?;

        Ok(violations)
    }

    /// Verify all pub items have rustdoc
    pub fn validate_pub_item_docs(&self) -> Result<Vec<DocumentationViolation>> {
        let mut violations = Vec::new();

        // Patterns for public items
        let pub_struct_pattern = compile_regex(r"pub\s+struct\s+([A-Z][a-zA-Z0-9_]*)")?;
        let pub_enum_pattern = compile_regex(r"pub\s+enum\s+([A-Z][a-zA-Z0-9_]*)")?;
        let pub_trait_pattern = compile_regex(r"pub\s+trait\s+([A-Z][a-zA-Z0-9_]*)")?;
        let pub_fn_pattern = compile_regex(r"pub\s+(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)")?;
        let example_pattern = compile_regex(r"#\s*Example")?;

        for_each_crate_file(
            &self.config,
            Some(LanguageId::Rust),
            |entry, _src_dir, _crate_name| {
                let path = &entry.absolute_path;
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
                        let Some(path_str) = path.to_str() else {
                            continue;
                        };

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
            },
        )?;

        Ok(violations)
    }

    /// Checks if a line has a documentation comment above it.
    ///
    /// Looks backwards from the item line to find a `///` doc comment,
    /// skipping attributes and empty lines.
    fn has_doc_comment(&self, lines: &[&str], item_line: usize) -> bool {
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
            if attr_pattern().is_match(lines[i]) {
                if i == 0 {
                    return false;
                }
                i -= 1;
                continue;
            }

            // Check for doc comment
            return doc_comment_pattern().is_match(lines[i]);
        }
    }

    /// Extracts the complete documentation comment section for an item.
    ///
    /// Collects all consecutive `///` lines above the item, skipping attributes,
    /// and returns them as a single string for analysis.
    fn get_doc_comment_section(&self, lines: &[&str], item_line: usize) -> String {
        if item_line == 0 {
            return String::new();
        }

        let mut doc_lines = Vec::new();
        let mut i = item_line - 1;

        loop {
            let line = lines[i];

            // Skip attributes
            if attr_pattern().is_match(line) {
                if i == 0 {
                    break;
                }
                i -= 1;
                continue;
            }

            // Collect doc comment
            if let Some(cap) = doc_comment_capture_pattern().captures(line) {
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

crate::impl_validator!(
    DocumentationValidator,
    "documentation",
    "Validates documentation standards"
);
