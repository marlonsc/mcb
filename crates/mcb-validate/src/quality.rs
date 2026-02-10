//! Code Quality Validation
//!
//! Validates code quality standards:
//! - No unwrap/expect in production code (AST-based detection)
//! - No panic!() in production code
//! - File size limits (500 lines)
//! - Pending-comment detection (T.O.D.O./F.I.X.M.E./X.X.X./H.A.C.K.)
//!
//! Phase 2 deliverable: QUAL001 (no-unwrap) detects `.unwrap()` calls via AST

use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::ast::UnwrapDetector;
use crate::thresholds::thresholds;
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};

/// Quality violation types representing specific code quality issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityViolation {
    /// Indicates usage of `unwrap()` in production code, which poses a panic risk.
    UnwrapInProduction {
        /// The file containing the violation.
        file: PathBuf,
        /// The line number where the `unwrap()` call was detected.
        line: usize,
        /// Contextual code snippet showing the usage of `unwrap()`.
        context: String,
        /// The severity level of the violation.
        severity: Severity,
    },
    /// Indicates usage of `expect()` in production code, which poses a panic risk.
    ExpectInProduction {
        /// The file containing the violation.
        file: PathBuf,
        /// The line number where the `expect()` call was detected.
        line: usize,
        /// Contextual code snippet showing the usage of `expect()`.
        context: String,
        /// The severity level of the violation.
        severity: Severity,
    },
    /// Indicates usage of `panic!()` macro in production code.
    PanicInProduction {
        /// The file containing the violation.
        file: PathBuf,
        /// The line number where the `panic!()` macro occurred.
        line: usize,
        /// Contextual code snippet showing the usage of `panic!()`.
        context: String,
        /// The severity level of the violation.
        severity: Severity,
    },
    /// Indicates a file that exceeds the maximum allowed line count.
    FileTooLarge {
        /// The file containing the violation.
        file: PathBuf,
        /// The current total number of lines in the file.
        lines: usize,
        /// The maximum allowed number of lines for this file.
        max_allowed: usize,
        /// The severity level of the violation.
        severity: Severity,
    },
    /// Indicates presence of pending task comments (tracked via `PENDING_LABEL_*` constants).
    TodoComment {
        /// The file containing the violation.
        file: PathBuf,
        /// The line number where the pending task comment was found.
        line: usize,
        /// The content of the comment, including the label type and message text.
        content: String,
        /// The severity level of the violation.
        severity: Severity,
    },
    /// Indicates usage of `allow(dead_code)` attribute, which is not permitted.
    DeadCodeAllowNotPermitted {
        /// The file containing the violation.
        file: PathBuf,
        /// The line number where the `#[allow(dead_code)]` attribute was found.
        line: usize,
        /// The name of the item (struct, function, field) marked as allowed dead code.
        item_name: String,
        /// The severity level of the violation.
        severity: Severity,
    },
    /// Indicates a struct field that is defined but never used.
    UnusedStructField {
        /// The file containing the violation.
        file: PathBuf,
        /// The line number where the field is defined.
        line: usize,
        /// The name of the struct containing the unused field.
        struct_name: String,
        /// The name of the field that appears to be unused.
        field_name: String,
        /// The severity level of the violation.
        severity: Severity,
    },
    /// Indicates a function that is marked as dead code and appears uncalled.
    DeadFunctionUncalled {
        /// The file containing the violation.
        file: PathBuf,
        /// The line number where the function is defined.
        line: usize,
        /// The name of the function that appears to be dead/uncalled.
        function_name: String,
        /// The severity level of the violation.
        severity: Severity,
    },
}

impl QualityViolation {
    /// Returns the severity level of the violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}

#[allow(clippy::too_many_lines)]
impl std::fmt::Display for QualityViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnwrapInProduction {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "unwrap() in production: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
            Self::ExpectInProduction {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "expect() in production: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
            Self::PanicInProduction {
                file,
                line,
                context,
                ..
            } => {
                write!(
                    f,
                    "panic!() in production: {}:{} - {}",
                    file.display(),
                    line,
                    context
                )
            }
            Self::FileTooLarge {
                file,
                lines,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "File too large: {} has {} lines (max: {})",
                    file.display(),
                    lines,
                    max_allowed
                )
            }
            Self::TodoComment {
                file,
                line,
                content,
                ..
            } => {
                write!(f, "Pending: {}:{} - {}", file.display(), line, content)
            }
            Self::DeadCodeAllowNotPermitted {
                file,
                line,
                item_name,
                ..
            } => {
                write!(
                    f,
                    "{}:{} - {} (allow(dead_code) not permitted)",
                    file.display(),
                    line,
                    item_name
                )
            }
            Self::UnusedStructField {
                file,
                line,
                struct_name,
                field_name,
                ..
            } => {
                write!(
                    f,
                    "Unused struct field: {}:{} - Field '{}' in struct '{}' is unused",
                    file.display(),
                    line,
                    field_name,
                    struct_name
                )
            }
            Self::DeadFunctionUncalled {
                file,
                line,
                function_name,
                ..
            } => {
                write!(
                    f,
                    "Dead function: {}:{} - Function '{}' marked as dead but appears uncalled",
                    file.display(),
                    line,
                    function_name
                )
            }
        }
    }
}

impl Violation for QualityViolation {
    fn id(&self) -> &str {
        match self {
            Self::UnwrapInProduction { .. } => "QUAL001",
            Self::ExpectInProduction { .. } => "QUAL002",
            Self::PanicInProduction { .. } => "QUAL003",
            Self::FileTooLarge { .. } => "QUAL004",
            Self::TodoComment { .. } => "QUAL005",
            Self::DeadCodeAllowNotPermitted { .. } => "QUAL020",
            Self::UnusedStructField { .. } => "QUAL021",
            Self::DeadFunctionUncalled { .. } => "QUAL022",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Quality
    }

    fn severity(&self) -> Severity {
        match self {
            Self::UnwrapInProduction { severity, .. }
            | Self::ExpectInProduction { severity, .. }
            | Self::PanicInProduction { severity, .. }
            | Self::FileTooLarge { severity, .. }
            | Self::TodoComment { severity, .. }
            | Self::DeadCodeAllowNotPermitted { severity, .. }
            | Self::UnusedStructField { severity, .. }
            | Self::DeadFunctionUncalled { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&std::path::PathBuf> {
        match self {
            Self::UnwrapInProduction { file, .. }
            | Self::ExpectInProduction { file, .. }
            | Self::PanicInProduction { file, .. }
            | Self::FileTooLarge { file, .. }
            | Self::TodoComment { file, .. }
            | Self::DeadCodeAllowNotPermitted { file, .. }
            | Self::UnusedStructField { file, .. }
            | Self::DeadFunctionUncalled { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::UnwrapInProduction { line, .. }
            | Self::ExpectInProduction { line, .. }
            | Self::PanicInProduction { line, .. }
            | Self::TodoComment { line, .. }
            | Self::DeadCodeAllowNotPermitted { line, .. }
            | Self::UnusedStructField { line, .. }
            | Self::DeadFunctionUncalled { line, .. } => Some(*line),
            Self::FileTooLarge { .. } => None,
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::UnwrapInProduction { .. } | Self::ExpectInProduction { .. } => {
                Some("Use `?` operator or handle the error explicitly".to_string())
            }
            Self::PanicInProduction { .. } => {
                Some("Return an error instead of panicking".to_string())
            }
            Self::FileTooLarge { max_allowed, .. } => Some(format!(
                "Split file into smaller modules (max {max_allowed} lines)"
            )),
            Self::TodoComment { .. } => {
                Some("Address the pending comment or create an issue to track it".to_string())
            }
            Self::DeadCodeAllowNotPermitted { .. } => {
                Some("Remove #[allow(dead_code)] and fix or remove the dead code; justifications are not permitted".to_string())
            }
            Self::UnusedStructField { .. } => {
                Some("Remove the unused field or document why it's kept (e.g., for serialization format versioning)".to_string())
            }
            Self::DeadFunctionUncalled { .. } => {
                Some("Remove the dead function or connect it to the public API if it's intended for future use".to_string())
            }
        }
    }
}

/// Validates codebase against quality standards and rules.
pub struct QualityValidator {
    config: ValidationConfig,
    max_file_lines: usize,
    excluded_paths: Vec<String>,
}

impl QualityValidator {
    /// Check if a line has an ignore hint for a specific violation type
    fn has_ignore_hint(&self, line: &str, violation_type: &str) -> bool {
        line.contains(&format!("mcb-validate-ignore: {violation_type}"))
    }
    /// Creates a new instance of the quality validator for the given workspace.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Creates a new validator instance using a provided configuration.
    pub fn with_config(config: ValidationConfig) -> Self {
        // Load file configuration to get quality rules
        let file_config = crate::config::FileConfig::load(&config.workspace_root);
        Self {
            config,
            max_file_lines: thresholds().max_file_lines,
            excluded_paths: file_config.rules.quality.excluded_paths,
        }
    }

    /// Configures the maximum allowed lines per file.
    #[must_use]
    pub fn with_max_file_lines(mut self, max: usize) -> Self {
        self.max_file_lines = max;
        self
    }

    /// Executes all configured quality checks and returns any violations found.
    pub fn validate_all(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_no_unwrap_expect()?);
        violations.extend(self.validate_no_panic()?);
        violations.extend(self.validate_file_sizes()?);
        violations.extend(self.find_todo_comments()?);
        violations.extend(self.validate_dead_code_annotations()?);
        Ok(violations)
    }

    /// Scans for and reports usage of `allow(dead_code)` attributes.
    pub fn validate_dead_code_annotations(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        let dead_code_pattern = Regex::new(r"#\[allow\([^\)]*dead_code[^\)]*\)\]").unwrap();
        let struct_pattern = Regex::new(r"pub\s+struct\s+(\w+)").unwrap();
        let fn_pattern = Regex::new(r"(?:pub\s+)?fn\s+(\w+)").unwrap();
        let field_pattern = Regex::new(r"(?:pub\s+)?(\w+):\s+").unwrap();

        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                .filter(|e| !e.path().to_string_lossy().contains("/tests/"))
            {
                // Skip deleted files
                if !entry.path().exists() {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let lines: Vec<&str> = content.lines().collect();

                for (i, line) in lines.iter().enumerate() {
                    if dead_code_pattern.is_match(line) {
                        let item_name = self
                            .find_dead_code_item(
                                &lines,
                                i,
                                &struct_pattern,
                                &fn_pattern,
                                &field_pattern,
                            )
                            .unwrap_or_else(|| "allow(dead_code)".to_string());
                        violations.push(QualityViolation::DeadCodeAllowNotPermitted {
                            file: entry.path().to_path_buf(),
                            line: i + 1,
                            item_name,
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    fn find_dead_code_item(
        &self,
        lines: &[&str],
        start_idx: usize,
        struct_pattern: &Regex,
        fn_pattern: &Regex,
        field_pattern: &Regex,
    ) -> Option<String> {
        let end = std::cmp::min(start_idx + 5, lines.len());
        for line in lines.iter().take(end).skip(start_idx) {
            if let Some(captures) = struct_pattern.captures(line)
                && let Some(name) = captures.get(1)
            {
                return Some(format!("struct {}", name.as_str()));
            }

            if let Some(captures) = fn_pattern.captures(line)
                && let Some(name) = captures.get(1)
            {
                return Some(format!("fn {}", name.as_str()));
            }

            if let Some(captures) = field_pattern.captures(line)
                && let Some(name) = captures.get(1)
            {
                return Some(format!("field {}", name.as_str()));
            }
        }

        None
    }

    /// Scans production code for usage of `unwrap()` and `expect()` methods.
    ///
    /// Uses AST-based detection to accurately identify method calls while ignoring
    /// test files and allowed patterns.
    pub fn validate_no_unwrap_expect(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        let mut detector = UnwrapDetector::new()?;

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
                .filter(|e| {
                    // Skip test files
                    let path_str = e.path().to_string_lossy();
                    !path_str.contains("/tests/")
                        && !path_str.contains("/target/")
                        && !path_str.ends_with("_test.rs")
                        && !path_str.ends_with("test.rs")
                })
            {
                // Use AST-based detection
                let detections = detector.detect_in_file(entry.path())?;

                for detection in detections {
                    // Skip detections in test modules
                    if detection.in_test {
                        continue;
                    }

                    // Skip if context contains SAFETY justification or ignore hints
                    // (checked via Regex since AST doesn't capture comments easily)
                    let content = std::fs::read_to_string(entry.path())?;
                    let lines: Vec<&str> = content.lines().collect();
                    if detection.line > 0 && detection.line <= lines.len() {
                        let line_content = lines[detection.line - 1];

                        // Check for safety comments
                        if line_content.contains("// SAFETY:")
                            || line_content.contains("// safety:")
                        {
                            continue;
                        }

                        // Check for ignore hints around the detection
                        let mut has_ignore = false;

                        // Check current line
                        if self.has_ignore_hint(line_content, "lock_poisoning_recovery")
                            || self.has_ignore_hint(line_content, "hardcoded_fallback")
                        {
                            has_ignore = true;
                        }

                        // Check previous lines (up to 3 lines before)
                        if !has_ignore && detection.line > 1 {
                            for i in 1..=3.min(detection.line - 1) {
                                let check_line = lines[detection.line - 1 - i];
                                if self.has_ignore_hint(check_line, "lock_poisoning_recovery")
                                    || self.has_ignore_hint(check_line, "hardcoded_fallback")
                                {
                                    has_ignore = true;
                                    break;
                                }
                            }
                        }

                        // Check next lines (up to 3 lines after)
                        if !has_ignore && detection.line < lines.len() {
                            for i in 0..3.min(lines.len() - detection.line) {
                                let check_line = lines[detection.line + i];
                                if self.has_ignore_hint(check_line, "lock_poisoning_recovery")
                                    || self.has_ignore_hint(check_line, "hardcoded_fallback")
                                {
                                    has_ignore = true;
                                    break;
                                }
                            }
                        }

                        if has_ignore {
                            continue;
                        }

                        // Skip cases where we're handling system-level errors appropriately
                        // (e.g., lock poisoning, which is a legitimate use of expect())
                        if detection.method == "expect"
                            && (line_content.contains("lock poisoned")
                                || line_content.contains("Lock poisoned")
                                || line_content.contains("poisoned")
                                || line_content.contains("Mutex poisoned"))
                        {
                            continue;
                        }
                    }

                    // Create appropriate violation based on method type
                    match detection.method.as_str() {
                        "unwrap" => {
                            violations.push(QualityViolation::UnwrapInProduction {
                                file: entry.path().to_path_buf(),
                                line: detection.line,
                                context: detection.context,
                                severity: Severity::Error,
                            });
                        }
                        "expect" => {
                            violations.push(QualityViolation::ExpectInProduction {
                                file: entry.path().to_path_buf(),
                                line: detection.line,
                                context: detection.context,
                                severity: Severity::Error,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Scans production code for usage of the `panic!()` macro.
    pub fn validate_no_panic(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();
        let panic_pattern = Regex::new(r"panic!\s*\(").unwrap();

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                let mut in_test_module = false;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();

                    // Skip comments
                    if trimmed.starts_with("//") {
                        continue;
                    }

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Check for panic!
                    if panic_pattern.is_match(line) {
                        violations.push(QualityViolation::PanicInProduction {
                            file: entry.path().to_path_buf(),
                            line: line_num + 1,
                            context: trimmed.to_string(),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Checks that source files do not exceed the configured line count limit.
    pub fn validate_file_sizes(&self) -> Result<Vec<QualityViolation>> {
        let mut violations = Vec::new();

        // Use get_scan_dirs() for proper handling of both crate-style and flat directories
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| {
                    e.path().extension().is_some_and(|ext| ext == "rs")
                        && !self.config.should_exclude(e.path())
                        && !e.path().to_string_lossy().contains("/tests/")
                        && !e.path().to_string_lossy().contains("/target/")
                        && !e.path().to_string_lossy().ends_with("_test.rs")
                })
            {
                // Skip deleted files
                if !entry.path().exists() {
                    continue;
                }

                let path_str = entry.path().to_string_lossy();

                // Skip paths excluded in configuration (e.g., large vector store implementations)
                if self
                    .excluded_paths
                    .iter()
                    .any(|excluded| path_str.contains(excluded.as_str()))
                {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let line_count = content.lines().count();

                if line_count > self.max_file_lines {
                    violations.push(QualityViolation::FileTooLarge {
                        file: entry.path().to_path_buf(),
                        lines: line_count,
                        max_allowed: self.max_file_lines,
                        severity: Severity::Warning,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Scans for pending task comments matching `PENDING_LABEL_*` constants.
    pub fn find_todo_comments(&self) -> Result<Vec<QualityViolation>> {
        use crate::constants::{
            PENDING_LABEL_FIXME, PENDING_LABEL_HACK, PENDING_LABEL_TODO, PENDING_LABEL_XXX,
        };
        let todo_pattern = Regex::new(&format!(
            r"(?i)({PENDING_LABEL_TODO}|{PENDING_LABEL_FIXME}|{PENDING_LABEL_XXX}|{PENDING_LABEL_HACK}):?\s*(.*)"
        ))
        .unwrap();

        let mut violations = Vec::new();
        for src_dir in self.config.get_scan_dirs()? {
            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip deleted files - check existence before reading
                if !entry.path().exists() {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;

                for (line_num, line) in content.lines().enumerate() {
                    if let Some(cap) = todo_pattern.captures(line) {
                        let todo_type = cap.get(1).map_or("", |m| m.as_str());
                        let message = cap.get(2).map_or("", |m| m.as_str()).trim();

                        violations.push(QualityViolation::TodoComment {
                            file: entry.path().to_path_buf(),
                            line: line_num + 1,
                            content: format!("{}: {}", todo_type.to_uppercase(), message),
                            severity: Severity::Info,
                        });
                    }
                }
            }
        }

        Ok(violations)
    }
}

impl_validator!(
    QualityValidator,
    "quality",
    "Validates code quality (no unwrap/expect)"
);
