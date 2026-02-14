//! Async Pattern Validation
//!
//! Detects async-specific anti-patterns based on Tokio documentation:
//! - Blocking in async (`std::thread::sleep`, `std::sync::Mutex` in async)
//! - `block_on()` in async context
//! - Spawn patterns (missing `JoinHandle` handling)
//! - Wrong mutex types in async code

use std::path::PathBuf;

use crate::pattern_registry::{compile_regex_triples, compile_regexes, required_pattern};
use crate::scan::for_each_scan_rs_path;
use crate::traits::violation::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};

crate::define_violations! {
    dynamic_severity,
    ViolationCategory::Async,
    pub enum AsyncViolation {
        /// Blocking call in async function
        #[violation(
            id = "ASYNC001",
            severity = Warning,
            message = "Blocking in async: {file}:{line} - {blocking_call} ({suggestion})",
            suggestion = "{suggestion}"
        )]
        BlockingInAsync {
            file: PathBuf,
            line: usize,
            blocking_call: String,
            suggestion: String,
            severity: Severity,
        },
        /// `block_on()` used in async context
        #[violation(
            id = "ASYNC002",
            severity = Warning,
            message = "block_on in async: {file}:{line} - {context}",
            suggestion = "Use .await instead of block_on() in async context"
        )]
        BlockOnInAsync {
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
        /// `std::sync::Mutex` used in async code (should use `tokio::sync::Mutex`)
        #[violation(
            id = "ASYNC003",
            severity = Warning,
            message = "Wrong mutex type: {file}:{line} - {mutex_type} ({suggestion})",
            suggestion = "{suggestion}"
        )]
        WrongMutexType {
            file: PathBuf,
            line: usize,
            mutex_type: String,
            suggestion: String,
            severity: Severity,
        },
        /// Spawn without awaiting `JoinHandle`
        #[violation(
            id = "ASYNC004",
            severity = Info,
            message = "Unawaited spawn: {file}:{line} - {context}",
            suggestion = "Assign JoinHandle to a variable or use let _ = to explicitly ignore"
        )]
        UnawaitedSpawn {
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
    }
}

/// Helper methods for async violations.
impl AsyncViolation {
    /// Returns the severity level of this violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}

/// Async pattern validator
pub struct AsyncPatternValidator {
    config: ValidationConfig,
}

impl AsyncPatternValidator {
    /// Create a new async pattern validator
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Create a validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        Self { config }
    }

    /// Run all async validations
    pub fn validate_all(&self) -> Result<Vec<AsyncViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_blocking_in_async()?);
        violations.extend(self.validate_block_on_usage()?);
        violations.extend(self.validate_mutex_types()?);
        violations.extend(self.validate_spawn_patterns()?);
        Ok(violations)
    }

    /// Detect blocking calls in async functions
    #[allow(clippy::too_many_lines)]
    pub fn validate_blocking_in_async(&self) -> Result<Vec<AsyncViolation>> {
        let mut violations = Vec::new();

        let async_fn_pattern = required_pattern("ASYNC001.async_fn_named")?;

        let blocking_patterns = [
            (
                r"std::thread::sleep",
                "std::thread::sleep",
                "Use tokio::time::sleep instead",
            ),
            (
                r"thread::sleep",
                "thread::sleep",
                "Use tokio::time::sleep instead",
            ),
            (
                r"std::fs::read",
                "std::fs::read",
                "Use tokio::fs::read instead",
            ),
            (
                r"std::fs::write",
                "std::fs::write",
                "Use tokio::fs::write instead",
            ),
            (
                r"std::fs::File::open",
                "std::fs::File::open",
                "Use tokio::fs::File::open instead",
            ),
            (
                r"std::fs::File::create",
                "std::fs::File::create",
                "Use tokio::fs::File::create instead",
            ),
            (
                r"\.blocking_lock\(\)",
                ".blocking_lock()",
                "Use .lock().await instead in async context",
            ),
            (
                r"\.blocking_read\(\)",
                ".blocking_read()",
                "Use .read().await instead in async context",
            ),
            (
                r"\.blocking_write\(\)",
                ".blocking_write()",
                "Use .write().await instead in async context",
            ),
        ];

        let compiled_blocking = compile_regex_triples(&blocking_patterns)?;

        for_each_scan_rs_path(&self.config, false, |path, _src_dir| {
            if path.to_string_lossy().contains("/tests/") {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_async_fn = false;
            let mut async_fn_depth = 0;
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

                // Track async function entry
                if async_fn_pattern.is_match(trimmed) {
                    in_async_fn = true;
                    async_fn_depth = 0;
                }

                if in_async_fn {
                    let open = line.chars().filter(|c| *c == '{').count();
                    let close = line.chars().filter(|c| *c == '}').count();
                    async_fn_depth += i32::try_from(open).unwrap_or(i32::MAX);
                    async_fn_depth -= i32::try_from(close).unwrap_or(i32::MAX);

                    // Check for blocking calls
                    for (pattern, desc, sugg) in &compiled_blocking {
                        if pattern.is_match(line) {
                            violations.push(AsyncViolation::BlockingInAsync {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                blocking_call: desc.to_string(),
                                suggestion: sugg.to_string(),
                                severity: Severity::Warning,
                            });
                        }
                    }

                    if async_fn_depth <= 0 {
                        in_async_fn = false;
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Detect `block_on()` usage in async context
    pub fn validate_block_on_usage(&self) -> Result<Vec<AsyncViolation>> {
        let mut violations = Vec::new();

        let async_fn_pattern = required_pattern("ASYNC001.async_fn")?;
        let block_on_patterns = [
            r"block_on\(",
            r"futures::executor::block_on",
            r"tokio::runtime::Runtime::new\(\).*\.block_on",
            r"Runtime::new\(\).*\.block_on",
        ];

        let compiled_block_on = compile_regexes(block_on_patterns)?;

        for_each_scan_rs_path(&self.config, false, |path, _src_dir| {
            if path.to_string_lossy().contains("/tests/") {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_async_fn = false;
            let mut async_fn_depth = 0;
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

                // Track async function entry
                if async_fn_pattern.is_match(trimmed) {
                    in_async_fn = true;
                    async_fn_depth = 0;
                }

                if in_async_fn {
                    let open = line.chars().filter(|c| *c == '{').count();
                    let close = line.chars().filter(|c| *c == '}').count();
                    async_fn_depth += i32::try_from(open).unwrap_or(i32::MAX);
                    async_fn_depth -= i32::try_from(close).unwrap_or(i32::MAX);

                    // Check for block_on calls
                    for pattern in &compiled_block_on {
                        if pattern.is_match(line) {
                            violations.push(AsyncViolation::BlockOnInAsync {
                                file: path.to_path_buf(),
                                line: line_num + 1,
                                context: trimmed.chars().take(80).collect(),
                                severity: Severity::Error,
                            });
                        }
                    }

                    if async_fn_depth <= 0 {
                        in_async_fn = false;
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Detect `std::sync::Mutex` usage in files with async code
    pub fn validate_mutex_types(&self) -> Result<Vec<AsyncViolation>> {
        let mut violations = Vec::new();

        let async_indicator = required_pattern("ASYNC001.async_indicator")?;
        let std_mutex_patterns = [
            (
                r"use\s+std::sync::Mutex",
                "std::sync::Mutex import",
                "Use tokio::sync::Mutex for async code",
            ),
            (
                r"std::sync::Mutex<",
                "std::sync::Mutex type",
                "Use tokio::sync::Mutex for async code",
            ),
            (
                r"use\s+std::sync::RwLock",
                "std::sync::RwLock import",
                "Use tokio::sync::RwLock for async code",
            ),
            (
                r"std::sync::RwLock<",
                "std::sync::RwLock type",
                "Use tokio::sync::RwLock for async code",
            ),
        ];

        let compiled_mutex = compile_regex_triples(&std_mutex_patterns)?;

        for_each_scan_rs_path(&self.config, false, |path, _src_dir| {
            if path.to_string_lossy().contains("/tests/") {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;

            // Only check files that have async code
            if !async_indicator.is_match(&content) {
                return Ok(());
            }

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

                // Check for std mutex patterns
                for (pattern, desc, sugg) in &compiled_mutex {
                    if pattern.is_match(line) {
                        violations.push(AsyncViolation::WrongMutexType {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            mutex_type: desc.to_string(),
                            suggestion: sugg.to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }

    /// Detect spawn without await patterns
    pub fn validate_spawn_patterns(&self) -> Result<Vec<AsyncViolation>> {
        let mut violations = Vec::new();

        // Pattern: tokio::spawn without assigning to variable or awaiting
        let spawn_pattern = required_pattern("ASYNC001.tokio_spawn")?;
        let assigned_spawn_pattern = required_pattern("ASYNC001.assigned_spawn")?;
        let fn_pattern = required_pattern("ASYNC001.fn_decl")?;

        // Function name patterns that indicate intentional fire-and-forget spawns
        // Includes constructor patterns that often spawn background workers
        let background_fn_patterns = [
            "spawn",
            "background",
            "graceful",
            "shutdown",
            "start",
            "run",
            "worker",
            "daemon",
            "listener",
            "handler",
            "process",
            "new",
            "with_",
            "init",
            "create",
            "build", // Constructor patterns
        ];

        for_each_scan_rs_path(&self.config, false, |path, _src_dir| {
            if path.to_string_lossy().contains("/tests/") {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_test_module = false;
            let mut current_fn_name = String::new();

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

                // Track current function name
                if let Some(cap) = fn_pattern.captures(line) {
                    current_fn_name = cap.get(1).map_or("", |m| m.as_str()).to_lowercase();
                }

                // Check for unassigned spawn
                if spawn_pattern.is_match(line) && !assigned_spawn_pattern.is_match(line) {
                    // Check if it's being used in a chain (e.g., .await)
                    if !line.contains(".await") && !line.contains("let _") {
                        // Skip if function name suggests fire-and-forget is intentional
                        let is_background_fn = background_fn_patterns
                            .iter()
                            .any(|p| current_fn_name.contains(p));
                        if is_background_fn {
                            continue;
                        }
                        violations.push(AsyncViolation::UnawaitedSpawn {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            context: trimmed.chars().take(80).collect(),
                            severity: Severity::Info,
                        });
                    }
                }
            }

            Ok(())
        })?;

        Ok(violations)
    }
}

crate::impl_validator!(
    AsyncPatternValidator,
    "async_patterns",
    "Validates async patterns (blocking calls, mutex types, spawn patterns)"
);
