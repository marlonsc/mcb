//! Performance Pattern Validation
//!
//! Detects performance anti-patterns that PMAT and Clippy might miss:
//! - Clone abuse (redundant clones, clones in loops)
//! - Allocation patterns (Vec/String in loops)
//! - Arc/Mutex overuse
//! - Inefficient iterator patterns

use std::path::PathBuf;

use regex::Regex;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::config::PerformanceRulesConfig;
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

    /// Detect .`clone()` calls inside loops
    pub fn validate_clone_in_loops(&self) -> Result<Vec<PerformanceViolation>> {
        let mut violations = Vec::new();

        let loop_start_pattern = Regex::new(r"^\s*(for|while|loop)\s+").unwrap();
        let clone_pattern = Regex::new(r"\.clone\(\)").unwrap();

        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }
            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
                    continue;
                }

                // Skip routing files - clone is necessary to return owned values from borrowed refs
                let path_str = entry.path().to_string_lossy();
                if path_str.ends_with("/router.rs") || path_str.contains("/routing/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut in_loop = false;
                let mut loop_depth = 0;
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

                    // Track loop entry
                    if loop_start_pattern.is_match(trimmed) {
                        in_loop = true;
                        loop_depth = 0;
                    }

                    if in_loop {
                        let o = line.chars().filter(|c| *c == '{').count();
                        let c = line.chars().filter(|c| *c == '}').count();
                        loop_depth += i32::try_from(o).unwrap_or(i32::MAX);
                        loop_depth -= i32::try_from(c).unwrap_or(i32::MAX);

                        if clone_pattern.is_match(line) {
                            // Skip if it's a method definition
                            if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
                                continue;
                            }
                            // Skip struct field initialization patterns (field: value.clone())
                            // These are typically required for ownership transfer
                            if trimmed.contains(": ") && trimmed.ends_with(".clone(),") {
                                continue;
                            }
                            // Skip let bindings that clone for the loop (setup pattern)
                            if trimmed.starts_with("let ") && trimmed.contains("= ") {
                                continue;
                            }
                            // Skip insert patterns (common in HashMap operations)
                            if trimmed.contains(".insert(") {
                                continue;
                            }
                            // Skip push patterns (common in Vec operations)
                            if trimmed.contains(".push(") {
                                continue;
                            }
                            violations.push(PerformanceViolation::CloneInLoop {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                context: trimmed.chars().take(80).collect(),
                                suggestion: "Consider borrowing or moving instead of cloning"
                                    .to_string(),
                                severity: Severity::Warning,
                            });
                        }

                        if loop_depth <= 0 {
                            in_loop = false;
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect `Vec::new()` or `String::new()` inside loops
    pub fn validate_allocation_in_loops(&self) -> Result<Vec<PerformanceViolation>> {
        let mut violations = Vec::new();

        let loop_start_pattern = Regex::new(r"^\s*(for|while|loop)\s+").unwrap();
        let allocation_patterns = [
            (r"Vec::new\(\)", "Vec::new()"),
            (r"Vec::with_capacity\(", "Vec::with_capacity"),
            (r"String::new\(\)", "String::new()"),
            (r"String::with_capacity\(", "String::with_capacity"),
            (r"HashMap::new\(\)", "HashMap::new()"),
            (r"HashSet::new\(\)", "HashSet::new()"),
        ];

        let compiled_patterns: Vec<_> = allocation_patterns
            .iter()
            .filter_map(|(p, desc)| Regex::new(p).ok().map(|r| (r, *desc)))
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }
            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
                    continue;
                }

                let content = std::fs::read_to_string(entry.path())?;
                let mut in_loop = false;
                let mut loop_depth = 0;
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

                    // Track loop entry
                    if loop_start_pattern.is_match(trimmed) {
                        in_loop = true;
                        loop_depth = 0;
                    }

                    if in_loop {
                        let o = line.chars().filter(|c| *c == '{').count();
                        let c = line.chars().filter(|c| *c == '}').count();
                        loop_depth += i32::try_from(o).unwrap_or(i32::MAX);
                        loop_depth -= i32::try_from(c).unwrap_or(i32::MAX);

                        // Check for allocations in loop
                        for (pattern, alloc_type) in &compiled_patterns {
                            if pattern.is_match(line) {
                                violations.push(PerformanceViolation::AllocationInLoop {
                                    file: entry.path().to_path_buf(),
                                    line: line_num + 1,
                                    allocation_type: alloc_type.to_string(),
                                    suggestion: "Move allocation outside loop or reuse buffer"
                                        .to_string(),
                                    severity: Severity::Warning,
                                });
                            }
                        }

                        if loop_depth <= 0 {
                            in_loop = false;
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect Arc/Mutex overuse patterns
    pub fn validate_arc_mutex_overuse(&self) -> Result<Vec<PerformanceViolation>> {
        let mut violations = Vec::new();

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

        let compiled_patterns: Vec<_> = overuse_patterns
            .iter()
            .filter_map(|(p, desc, sugg)| Regex::new(p).ok().map(|r| (r, *desc, *sugg)))
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }
            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
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

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Check for overuse patterns
                    for (pattern, desc, sugg) in &compiled_patterns {
                        if pattern.is_match(line) {
                            violations.push(PerformanceViolation::ArcMutexOveruse {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                pattern: desc.to_string(),
                                suggestion: sugg.to_string(),
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect inefficient iterator patterns
    pub fn validate_inefficient_iterators(&self) -> Result<Vec<PerformanceViolation>> {
        let mut violations = Vec::new();

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

        let compiled_patterns: Vec<_> = inefficient_patterns
            .iter()
            .filter_map(|(p, desc, sugg)| Regex::new(p).ok().map(|r| (r, *desc, *sugg)))
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }
            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
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

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Check for inefficient patterns
                    for (pattern, desc, sugg) in &compiled_patterns {
                        if pattern.is_match(line) {
                            violations.push(PerformanceViolation::InefficientIterator {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                pattern: desc.to_string(),
                                suggestion: sugg.to_string(),
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }

    /// Detect inefficient string handling patterns
    pub fn validate_inefficient_strings(&self) -> Result<Vec<PerformanceViolation>> {
        let mut violations = Vec::new();

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

        let compiled_patterns: Vec<_> = inefficient_patterns
            .iter()
            .filter_map(|(p, desc, sugg)| Regex::new(p).ok().map(|r| (r, *desc, *sugg)))
            .collect();

        for src_dir in self.config.get_scan_dirs()? {
            if self.should_skip_crate(&src_dir) {
                continue;
            }
            for entry in WalkDir::new(&src_dir)
                .follow_links(false)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
            {
                // Skip test files
                if entry.path().to_string_lossy().contains("/tests/") {
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

                    // Track test modules
                    if trimmed.contains("#[cfg(test)]") {
                        in_test_module = true;
                        continue;
                    }

                    if in_test_module {
                        continue;
                    }

                    // Check for inefficient patterns
                    for (pattern, desc, sugg) in &compiled_patterns {
                        if pattern.is_match(line) {
                            violations.push(PerformanceViolation::InefficientString {
                                file: entry.path().to_path_buf(),
                                line: line_num + 1,
                                pattern: desc.to_string(),
                                suggestion: sugg.to_string(),
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
        }

        Ok(violations)
    }
    /// Check if a crate should be skipped based on configuration
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

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

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
name = "{name}"
version = "0.1.1"
"#
            ),
        )
        .unwrap();
    }

    #[test]
    fn test_clone_in_loop_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r"
pub fn process_items(items: Vec<String>) {
    for item in &items {
        // Direct clone in function call - detectable pattern
        process(item.clone());
    }
}
",
        );

        let validator = PerformanceValidator::new(temp.path());
        let violations = validator.validate_clone_in_loops().unwrap();

        assert!(!violations.is_empty(), "Should detect clone in loop");
    }

    #[test]
    fn test_allocation_in_loop_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r"
pub fn process_many() {
    for i in 0..100 {
        let mut v = Vec::new();
        v.push(i);
    }
}
",
        );

        let validator = PerformanceValidator::new(temp.path());
        let violations = validator.validate_allocation_in_loops().unwrap();

        assert!(!violations.is_empty(), "Should detect Vec::new() in loop");
    }

    #[test]
    fn test_arc_mutex_overuse_detection() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r"
use std::sync::Mutex;

pub struct Counter {
    value: Mutex<bool>,
}
",
        );

        let validator = PerformanceValidator::new(temp.path());
        let violations = validator.validate_arc_mutex_overuse().unwrap();

        assert!(!violations.is_empty(), "Should detect Mutex<bool>");
    }

    #[test]
    fn test_test_module_exemption() {
        let temp = TempDir::new().unwrap();
        create_test_crate(
            &temp,
            "mcb-test",
            r#"
pub fn good_code() {}

#[cfg(test)]
mod tests {
    fn test_clone() {
        for i in 0..10 {
            let x = "hello".to_string().clone();
        }
    }
}
"#,
        );

        let validator = PerformanceValidator::new(temp.path());
        let violations = validator.validate_clone_in_loops().unwrap();

        assert!(
            violations.is_empty(),
            "Test modules should be exempt: {violations:?}"
        );
    }
}
