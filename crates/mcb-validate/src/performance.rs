//! Performance Pattern Validation
//!
//! This module provides the `PerformanceValidator` which identifies common performance
//! anti-patterns in Rust code. It focuses on identifying clone abuse, unnecessary
//! allocations in loops, and suboptimal synchronization patterns.
//!
//! # Code Smells
//!
//! Detects performance anti-patterns that PMAT and Clippy might miss:
//! - Clone abuse (redundant clones, clones in loops)
//! - Allocation patterns (Vec/String in loops)
//! - Arc/Mutex overuse
//! - Inefficient iterator patterns

use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::config::PerformanceRulesConfig;
use crate::pattern_registry::{compile_regex, compile_regex_triples, compile_regexes};
use crate::scan::for_each_scan_rs_path;
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};

/// Performance violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceViolation {
    /// .`clone()` called inside a loop
    CloneInLoop {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Code context showing the clone call.
        context: String,
        /// Suggested improvement.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Vec/String allocation inside a loop
    AllocationInLoop {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Type of allocation detected (e.g., "Vec::new()").
        allocation_type: String,
        /// Suggested improvement.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// `Arc<Mutex<T>>` where simpler patterns would work
    ArcMutexOveruse {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// The overuse pattern detected.
        pattern: String,
        /// Suggested alternative.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Inefficient iterator pattern
    InefficientIterator {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// The inefficient pattern detected.
        pattern: String,
        /// Suggested optimized pattern.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Inefficient string handling
    InefficientString {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// The inefficient string pattern detected.
        pattern: String,
        /// Suggested optimization.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
}

impl PerformanceViolation {
    /// Returns the severity level of the violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}

/// Display implementation for performance violations.
///
/// Formats violations as human-readable messages with file location, line number,
/// and context about the performance issue detected.
impl std::fmt::Display for PerformanceViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CloneInLoop {
                file,
                line,
                context,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Clone in loop: {}:{} - {} ({})",
                    file.display(),
                    line,
                    context,
                    suggestion
                )
            }
            Self::AllocationInLoop {
                file,
                line,
                allocation_type,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Allocation in loop: {}:{} - {} ({})",
                    file.display(),
                    line,
                    allocation_type,
                    suggestion
                )
            }
            Self::ArcMutexOveruse {
                file,
                line,
                pattern,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Arc/Mutex overuse: {}:{} - {} ({})",
                    file.display(),
                    line,
                    pattern,
                    suggestion
                )
            }
            Self::InefficientIterator {
                file,
                line,
                pattern,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Inefficient iterator: {}:{} - {} ({})",
                    file.display(),
                    line,
                    pattern,
                    suggestion
                )
            }
            Self::InefficientString {
                file,
                line,
                pattern,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Inefficient string: {}:{} - {} ({})",
                    file.display(),
                    line,
                    pattern,
                    suggestion
                )
            }
        }
    }
}

/// Violation trait implementation for performance violations.
///
/// Provides violation metadata including ID, category, severity, file location,
/// line number, and remediation suggestions for performance anti-patterns.
impl Violation for PerformanceViolation {
    fn id(&self) -> &str {
        match self {
            Self::CloneInLoop { .. } => "PERF001",
            Self::AllocationInLoop { .. } => "PERF002",
            Self::ArcMutexOveruse { .. } => "PERF003",
            Self::InefficientIterator { .. } => "PERF004",
            Self::InefficientString { .. } => "PERF005",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Performance
    }

    fn severity(&self) -> Severity {
        match self {
            Self::CloneInLoop { severity, .. }
            | Self::AllocationInLoop { severity, .. }
            | Self::ArcMutexOveruse { severity, .. }
            | Self::InefficientIterator { severity, .. }
            | Self::InefficientString { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::CloneInLoop { file, .. }
            | Self::AllocationInLoop { file, .. }
            | Self::ArcMutexOveruse { file, .. }
            | Self::InefficientIterator { file, .. }
            | Self::InefficientString { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::CloneInLoop { line, .. }
            | Self::AllocationInLoop { line, .. }
            | Self::ArcMutexOveruse { line, .. }
            | Self::InefficientIterator { line, .. }
            | Self::InefficientString { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::CloneInLoop { suggestion, .. }
            | Self::AllocationInLoop { suggestion, .. }
            | Self::ArcMutexOveruse { suggestion, .. }
            | Self::InefficientIterator { suggestion, .. }
            | Self::InefficientString { suggestion, .. } => Some(suggestion.clone()),
        }
    }
}

/// Performance pattern validator
pub struct PerformanceValidator {
    config: ValidationConfig,
    rules: PerformanceRulesConfig,
}

impl PerformanceValidator {
    /// Create a new performance validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        let root: PathBuf = workspace_root.into();
        let file_config = crate::config::FileConfig::load(&root);
        Self::with_config(ValidationConfig::new(root), &file_config.rules.performance)
    }

    /// Create a validator with custom configuration
    pub fn with_config(config: ValidationConfig, rules: &PerformanceRulesConfig) -> Self {
        Self {
            config,
            rules: rules.clone(),
        }
    }

    /// Run all performance validations
    pub fn validate_all(&self) -> Result<Vec<PerformanceViolation>> {
        if !self.rules.enabled {
            return Ok(Vec::new());
        }
        let mut violations = Vec::new();
        violations.extend(self.validate_clone_in_loops()?);
        violations.extend(self.validate_allocation_in_loops()?);
        violations.extend(self.validate_arc_mutex_overuse()?);
        violations.extend(self.validate_inefficient_iterators()?);
        violations.extend(self.validate_inefficient_strings()?);
        Ok(violations)
    }

    /// Detect .`clone()` calls inside loops.
    pub fn validate_clone_in_loops(&self) -> Result<Vec<PerformanceViolation>> {
        let clone_pattern = compile_regex(r"\.clone\(\)")?;
        self.scan_files_with_patterns_in_loops(&[clone_pattern], |file, line_num, line| {
            let trimmed = line.trim();
            if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
                return None;
            }
            if trimmed.starts_with("let ") {
                return None;
            }
            if trimmed.contains(".insert(") {
                return None;
            }
            if trimmed.contains(": ") && trimmed.ends_with(".clone(),") {
                return None;
            }
            Some(PerformanceViolation::CloneInLoop {
                file,
                line: line_num,
                context: line.trim().chars().take(80).collect(),
                suggestion: "Consider borrowing or moving instead of cloning".to_string(),
                severity: Severity::Warning,
            })
        })
    }

    /// Helper: Scan files for patterns inside loops
    fn scan_files_with_patterns_in_loops<F>(
        &self,
        patterns: &[Regex],
        make_violation: F,
    ) -> Result<Vec<PerformanceViolation>>
    where
        F: Fn(PathBuf, usize, &str) -> Option<PerformanceViolation>,
    {
        let mut violations = Vec::new();
        let loop_start_pattern = compile_regex(r"^\s*(for|while|loop)\s+")?;

        for_each_scan_rs_path(&self.config, false, |path, src_dir| {
            if self.should_skip_crate(src_dir) || path.to_string_lossy().contains("/tests/") {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();
            let mut i = 0;

            while i < lines.len() {
                let trimmed = lines[i].trim();

                if !trimmed.starts_with("//")
                    && loop_start_pattern.is_match(trimmed)
                    && let Some((block_lines, end_idx)) =
                        crate::scan::extract_balanced_block(&lines, i)
                {
                    for (offset, line) in block_lines.iter().enumerate() {
                        let line_num = i + offset;
                        for pattern in patterns {
                            if pattern.is_match(line)
                                && let Some(violation) =
                                    make_violation(path.to_path_buf(), line_num + 1, line)
                            {
                                violations.push(violation);
                            }
                        }
                    }
                    i = end_idx + 1;
                    continue;
                }
                i += 1;
            }
            Ok(())
        })?;

        Ok(violations)
    }

    /// Detect `Vec::new()` or `String::new()` inside loops.
    pub fn validate_allocation_in_loops(&self) -> Result<Vec<PerformanceViolation>> {
        let allocation_patterns = [
            r"Vec::new\(\)",
            r"Vec::with_capacity\(",
            r"String::new\(\)",
            r"String::with_capacity\(",
            r"HashMap::new\(\)",
            r"HashSet::new\(\)",
        ];

        let compiled_patterns = compile_regexes(allocation_patterns)?;

        self.scan_files_with_patterns_in_loops(&compiled_patterns, |file, line_num, line| {
            let allocation_type = if line.contains("Vec::") {
                "Vec allocation"
            } else if line.contains("String::") {
                "String allocation"
            } else if line.contains("HashMap::") {
                "HashMap allocation"
            } else if line.contains("HashSet::") {
                "HashSet allocation"
            } else {
                "Allocation"
            };

            Some(PerformanceViolation::AllocationInLoop {
                file,
                line: line_num,
                allocation_type: allocation_type.to_string(),
                suggestion: "Move allocation outside loop or reuse buffer".to_string(),
                severity: Severity::Warning,
            })
        })
    }

    /// Helper: Scan files and apply pattern matching with a custom violation builder.
    fn scan_files_with_patterns<F>(
        &self,
        compiled_patterns: &[(Regex, &str, &str)],
        make_violation: F,
    ) -> Result<Vec<PerformanceViolation>>
    where
        F: Fn(PathBuf, usize, &str, &str) -> PerformanceViolation,
    {
        let mut violations = Vec::new();

        for_each_scan_rs_path(&self.config, false, |path, src_dir| {
            if self.should_skip_crate(src_dir) || path.to_string_lossy().contains("/tests/") {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_test_module = false;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                if trimmed.starts_with("//") {
                    continue;
                }

                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    continue;
                }

                if in_test_module {
                    continue;
                }

                for (pattern, desc, sugg) in compiled_patterns {
                    if pattern.is_match(line) {
                        violations.push(make_violation(
                            path.to_path_buf(),
                            line_num + 1,
                            desc,
                            sugg,
                        ));
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Detect Arc/Mutex overuse patterns.
    pub fn validate_arc_mutex_overuse(&self) -> Result<Vec<PerformanceViolation>> {
        let overuse_patterns = [
            (r"Arc<Arc<", "Nested Arc<Arc<>>", "Use single Arc instead"),
            (r"Mutex<bool>", "Mutex<bool>", "Use AtomicBool instead"),
            (r"Mutex<usize>", "Mutex<usize>", "Use AtomicUsize instead"),
            (r"Mutex<u32>", "Mutex<u32>", "Use AtomicU32 instead"),
            (r"Mutex<u64>", "Mutex<u64>", "Use AtomicU64 instead"),
            (r"Mutex<i32>", "Mutex<i32>", "Use AtomicI32 instead"),
            (r"Mutex<i64>", "Mutex<i64>", "Use AtomicI64 instead"),
            (r"RwLock<bool>", "RwLock<bool>", "Use AtomicBool instead"),
        ];

        let compiled_patterns = compile_regex_triples(&overuse_patterns)?;

        self.scan_files_with_patterns(&compiled_patterns, |file, line, pattern, suggestion| {
            PerformanceViolation::ArcMutexOveruse {
                file,
                line,
                pattern: pattern.to_string(),
                suggestion: suggestion.to_string(),
                severity: Severity::Info,
            }
        })
    }

    /// Detect inefficient iterator patterns.
    pub fn validate_inefficient_iterators(&self) -> Result<Vec<PerformanceViolation>> {
        let inefficient_patterns = [
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

        let compiled_patterns = compile_regex_triples(&inefficient_patterns)?;

        self.scan_files_with_patterns(&compiled_patterns, |file, line, pattern, suggestion| {
            PerformanceViolation::InefficientIterator {
                file,
                line,
                pattern: pattern.to_string(),
                suggestion: suggestion.to_string(),
                severity: Severity::Info,
            }
        })
    }

    /// Detect inefficient string handling patterns.
    pub fn validate_inefficient_strings(&self) -> Result<Vec<PerformanceViolation>> {
        let inefficient_patterns = [
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

        let compiled_patterns = compile_regex_triples(&inefficient_patterns)?;

        self.scan_files_with_patterns(&compiled_patterns, |file, line, pattern, suggestion| {
            PerformanceViolation::InefficientString {
                file,
                line,
                pattern: pattern.to_string(),
                suggestion: suggestion.to_string(),
                severity: Severity::Info,
            }
        })
    }

    /// Check if a crate should be skipped based on configuration.
    fn should_skip_crate(&self, src_dir: &std::path::Path) -> bool {
        let path_str = src_dir.to_string_lossy();
        self.rules
            .excluded_crates
            .iter()
            .any(|excluded| path_str.contains(excluded))
    }
}

impl_validator!(
    PerformanceValidator,
    "performance",
    "Validates performance patterns (clones, allocations, Arc/Mutex usage)"
);
