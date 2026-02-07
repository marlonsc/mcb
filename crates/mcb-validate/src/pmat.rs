//! PMAT Integration Validator
//!
//! Integrates with PMAT CLI tool for additional analysis:
//! - Cyclomatic complexity analysis
//! - Dead code detection
//! - Technical Debt Gradient (TDG) scoring
//!
//! This validator is optional - it only runs if the `pmat` binary is available.

use crate::constants::{DEFAULT_COMPLEXITY_THRESHOLD, DEFAULT_TDG_THRESHOLD};
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// PMAT violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PmatViolation {
    /// High cyclomatic complexity
    HighComplexity {
        /// File path where violation occurred.
        file: PathBuf,
        /// Function name with high complexity.
        function: String,
        /// Measured cyclomatic complexity value.
        complexity: u32,
        /// Configured complexity threshold.
        threshold: u32,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Dead code detected
    DeadCode {
        /// File path where dead code was found.
        file: PathBuf,
        /// Line number of the dead code.
        line: usize,
        /// Type of dead code item (function, variable, etc).
        item_type: String,
        /// Name of the dead code item.
        name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Low TDG score (high technical debt)
    LowTdgScore {
        /// File path with low TDG score.
        file: PathBuf,
        /// Technical Debt Gradient score.
        score: u32,
        /// Configured TDG threshold.
        threshold: u32,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// PMAT tool not available
    PmatUnavailable {
        /// Error message explaining why PMAT is unavailable.
        message: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// PMAT execution error
    PmatError {
        /// Command that failed.
        command: String,
        /// Error message from PMAT execution.
        error: String,
        /// Severity level of the violation.
        severity: Severity,
    },
}

impl PmatViolation {
    /// Returns the severity level of this violation
    pub fn severity(&self) -> Severity {
        match self {
            Self::HighComplexity { severity, .. }
            | Self::DeadCode { severity, .. }
            | Self::LowTdgScore { severity, .. }
            | Self::PmatUnavailable { severity, .. }
            | Self::PmatError { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for PmatViolation {
    /// Formats the violation as a human-readable string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HighComplexity {
                file,
                function,
                complexity,
                threshold,
                ..
            } => {
                write!(
                    f,
                    "High complexity: {}::{} - complexity {} (threshold: {})",
                    file.display(),
                    function,
                    complexity,
                    threshold
                )
            }
            Self::DeadCode {
                file,
                line,
                item_type,
                name,
                ..
            } => {
                write!(
                    f,
                    "Dead code: {}:{} - {} '{}'",
                    file.display(),
                    line,
                    item_type,
                    name
                )
            }
            Self::LowTdgScore {
                file,
                score,
                threshold,
                ..
            } => {
                write!(
                    f,
                    "High technical debt: {} - TDG score {} (threshold: {})",
                    file.display(),
                    score,
                    threshold
                )
            }
            Self::PmatUnavailable { message, .. } => {
                write!(f, "PMAT unavailable: {message}")
            }
            Self::PmatError { command, error, .. } => {
                write!(f, "PMAT error running '{command}': {error}")
            }
        }
    }
}

impl Violation for PmatViolation {
    /// Returns the unique identifier for this violation type
    fn id(&self) -> &str {
        match self {
            Self::HighComplexity { .. } => "PMAT001",
            Self::DeadCode { .. } => "PMAT002",
            Self::LowTdgScore { .. } => "PMAT003",
            Self::PmatUnavailable { .. } => "PMAT004",
            Self::PmatError { .. } => "PMAT005",
        }
    }

    /// Returns the violation category
    fn category(&self) -> ViolationCategory {
        ViolationCategory::Pmat
    }

    /// Returns the severity level of this violation
    fn severity(&self) -> Severity {
        match self {
            Self::HighComplexity { severity, .. }
            | Self::DeadCode { severity, .. }
            | Self::LowTdgScore { severity, .. }
            | Self::PmatUnavailable { severity, .. }
            | Self::PmatError { severity, .. } => *severity,
        }
    }

    /// Returns the file path where the violation was detected
    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::HighComplexity { file, .. }
            | Self::DeadCode { file, .. }
            | Self::LowTdgScore { file, .. } => Some(file),
            Self::PmatUnavailable { .. } | Self::PmatError { .. } => None,
        }
    }

    /// Returns the line number where the violation was detected
    fn line(&self) -> Option<usize> {
        match self {
            Self::DeadCode { line, .. } => Some(*line),
            _ => None,
        }
    }

    /// Returns a suggestion for fixing this violation
    fn suggestion(&self) -> Option<String> {
        match self {
            Self::HighComplexity {
                function,
                complexity,
                threshold,
                ..
            } => Some(format!(
                "Consider refactoring '{function}' to reduce complexity from {complexity} to below {threshold}. \
                 Split into smaller functions or simplify control flow."
            )),
            Self::DeadCode {
                item_type, name, ..
            } => Some(format!("{item_type} {name}")),
            Self::LowTdgScore {
                score, threshold, ..
            } => Some(format!(
                "Technical debt score {score} exceeds threshold {threshold}. \
                 Address code smells, reduce complexity, and improve maintainability."
            )),
            Self::PmatUnavailable { .. } => {
                Some("Install PMAT CLI tool to enable additional analysis.".to_string())
            }
            Self::PmatError { command, .. } => Some(format!(
                "Check PMAT installation and run '{command}' manually to diagnose."
            )),
        }
    }
}

/// PMAT complexity result from JSON output
#[derive(Debug, Deserialize)]
struct ComplexityResult {
    /// File path containing the function
    #[serde(default)]
    file: Option<String>,
    /// Function name
    #[serde(default)]
    function: Option<String>,
    /// Cyclomatic complexity value
    #[serde(default)]
    complexity: Option<u32>,
}

/// PMAT dead code result from JSON output
#[derive(Debug, Deserialize)]
struct DeadCodeResult {
    /// File path containing the dead code
    #[serde(default)]
    file: Option<String>,
    /// Line number of the dead code
    #[serde(default)]
    line: Option<usize>,
    /// Type of item (function, struct, etc.)
    #[serde(default)]
    item_type: Option<String>,
    /// Name of the dead code item
    #[serde(default)]
    name: Option<String>,
}

/// PMAT TDG result from JSON output
#[derive(Debug, Deserialize)]
struct TdgResult {
    /// File path being analyzed
    #[serde(default)]
    file: Option<String>,
    /// Technical Debt Gradient score
    #[serde(default)]
    score: Option<u32>,
}

/// PMAT integration validator
pub struct PmatValidator {
    config: ValidationConfig,
    complexity_threshold: u32,
    tdg_threshold: u32,
    pmat_available: bool,
}

impl PmatValidator {
    /// Creates a new PMAT validator with default configuration
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Creates a validator with custom configuration
    pub fn with_config(config: ValidationConfig) -> Self {
        let pmat_available = Self::check_pmat_available();
        Self {
            config,
            complexity_threshold: DEFAULT_COMPLEXITY_THRESHOLD,
            tdg_threshold: DEFAULT_TDG_THRESHOLD,
            pmat_available,
        }
    }

    /// Sets the cyclomatic complexity threshold (builder pattern)
    #[must_use]
    pub fn with_complexity_threshold(mut self, threshold: u32) -> Self {
        self.complexity_threshold = threshold;
        self
    }

    /// Sets the Technical Debt Gradient threshold (builder pattern)
    #[must_use]
    pub fn with_tdg_threshold(mut self, threshold: u32) -> Self {
        self.tdg_threshold = threshold;
        self
    }

    /// Checks if the PMAT binary is available in the system PATH
    fn check_pmat_available() -> bool {
        Command::new("pmat")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Returns whether PMAT is available for this validator instance
    pub fn is_available(&self) -> bool {
        self.pmat_available
    }

    /// Runs all PMAT validations and returns detected violations
    pub fn validate_all(&self) -> Result<Vec<PmatViolation>> {
        let mut violations = Vec::new();

        if !self.pmat_available {
            violations.push(PmatViolation::PmatUnavailable {
                message: "pmat binary not found in PATH - skipping PMAT analysis".to_string(),
                severity: Severity::Info,
            });
            return Ok(violations);
        }

        violations.extend(self.validate_complexity()?);
        violations.extend(self.validate_dead_code()?);
        violations.extend(self.validate_tdg()?);

        Ok(violations)
    }

    /// Runs cyclomatic complexity analysis using PMAT
    pub fn validate_complexity(&self) -> Result<Vec<PmatViolation>> {
        let mut violations = Vec::new();

        if !self.pmat_available {
            return Ok(violations);
        }

        let workspace_root = &self.config.workspace_root;

        let output = Command::new("pmat")
            .args([
                "analyze",
                "complexity",
                "--project-path",
                workspace_root.to_str().unwrap_or("."),
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Try to parse as JSON array of complexity results
                if let Ok(results) = serde_json::from_str::<Vec<ComplexityResult>>(&stdout) {
                    for result in results {
                        if let (Some(file), Some(function), Some(complexity)) =
                            (result.file, result.function, result.complexity)
                            && complexity > self.complexity_threshold
                        {
                            violations.push(PmatViolation::HighComplexity {
                                file: PathBuf::from(file),
                                function,
                                complexity,
                                threshold: self.complexity_threshold,
                                severity: if complexity > self.complexity_threshold * 2 {
                                    Severity::Warning
                                } else {
                                    Severity::Info
                                },
                            });
                        }
                    }
                }
                // If parsing fails, that's OK - PMAT output format may vary
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                violations.push(PmatViolation::PmatError {
                    command: "pmat analyze complexity".to_string(),
                    error: stderr.to_string(),
                    severity: Severity::Info,
                });
            }
            Err(e) => {
                violations.push(PmatViolation::PmatError {
                    command: "pmat analyze complexity".to_string(),
                    error: e.to_string(),
                    severity: Severity::Info,
                });
            }
        }

        Ok(violations)
    }

    /// Runs dead code analysis using PMAT
    pub fn validate_dead_code(&self) -> Result<Vec<PmatViolation>> {
        let mut violations = Vec::new();

        if !self.pmat_available {
            return Ok(violations);
        }

        let workspace_root = &self.config.workspace_root;

        let output = Command::new("pmat")
            .args([
                "analyze",
                "dead-code",
                "--path",
                workspace_root.to_str().unwrap_or("."),
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Try to parse as JSON array of dead code results
                if let Ok(results) = serde_json::from_str::<Vec<DeadCodeResult>>(&stdout) {
                    for result in results {
                        if let (Some(file), Some(line), Some(item_type), Some(name)) =
                            (result.file, result.line, result.item_type, result.name)
                        {
                            violations.push(PmatViolation::DeadCode {
                                file: PathBuf::from(file),
                                line,
                                item_type,
                                name,
                                severity: Severity::Info,
                            });
                        }
                    }
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    violations.push(PmatViolation::PmatError {
                        command: "pmat analyze dead-code".to_string(),
                        error: stderr.to_string(),
                        severity: Severity::Info,
                    });
                }
            }
            Err(e) => {
                violations.push(PmatViolation::PmatError {
                    command: "pmat analyze dead-code".to_string(),
                    error: e.to_string(),
                    severity: Severity::Info,
                });
            }
        }

        Ok(violations)
    }

    /// Runs Technical Debt Gradient analysis using PMAT
    pub fn validate_tdg(&self) -> Result<Vec<PmatViolation>> {
        let mut violations = Vec::new();

        if !self.pmat_available {
            return Ok(violations);
        }

        let workspace_root = &self.config.workspace_root;

        let output = Command::new("pmat")
            .args(["tdg", workspace_root.to_str().unwrap_or(".")])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Try to parse as JSON array of TDG results
                if let Ok(results) = serde_json::from_str::<Vec<TdgResult>>(&stdout) {
                    for result in results {
                        if let (Some(file), Some(score)) = (result.file, result.score)
                            && score > self.tdg_threshold
                        {
                            violations.push(PmatViolation::LowTdgScore {
                                file: PathBuf::from(file),
                                score,
                                threshold: self.tdg_threshold,
                                severity: if score > self.tdg_threshold + 25 {
                                    Severity::Warning
                                } else {
                                    Severity::Info
                                },
                            });
                        }
                    }
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    violations.push(PmatViolation::PmatError {
                        command: "pmat tdg".to_string(),
                        error: stderr.to_string(),
                        severity: Severity::Info,
                    });
                }
            }
            Err(e) => {
                violations.push(PmatViolation::PmatError {
                    command: "pmat tdg".to_string(),
                    error: e.to_string(),
                    severity: Severity::Info,
                });
            }
        }

        Ok(violations)
    }
}

impl crate::validator_trait::Validator for PmatValidator {
    /// Returns the validator name
    fn name(&self) -> &'static str {
        "pmat"
    }

    /// Returns the validator description
    fn description(&self) -> &'static str {
        "PMAT integration for cyclomatic complexity, dead code detection, and TDG scoring"
    }

    /// Executes validation and returns violations as trait objects
    fn validate(&self, _config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate_all()?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }

    /// Returns whether this validator is enabled by default (only if PMAT is available)
    fn enabled_by_default(&self) -> bool {
        // Only enable by default if PMAT is available
        self.pmat_available
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pmat_availability_check() {
        // This test verifies the availability check works
        // It doesn't require PMAT to be installed
        let result = PmatValidator::check_pmat_available();
        // Result can be true or false depending on environment
        // Just verify the function completes without panic
        let _ = result;
    }

    #[test]
    fn test_validator_creation() {
        let temp = TempDir::new().unwrap();
        let validator = PmatValidator::new(temp.path());

        // Validator should be created successfully regardless of PMAT availability
        assert_eq!(validator.complexity_threshold, DEFAULT_COMPLEXITY_THRESHOLD);
        assert_eq!(validator.tdg_threshold, DEFAULT_TDG_THRESHOLD);
    }

    #[test]
    fn test_custom_thresholds() {
        let temp = TempDir::new().unwrap();
        let validator = PmatValidator::new(temp.path())
            .with_complexity_threshold(20)
            .with_tdg_threshold(60);

        assert_eq!(validator.complexity_threshold, 20);
        assert_eq!(validator.tdg_threshold, 60);
    }

    #[test]
    fn test_unavailable_pmat_returns_info() {
        let temp = TempDir::new().unwrap();
        let mut validator = PmatValidator::new(temp.path());

        // Force PMAT to be unavailable for testing
        validator.pmat_available = false;

        let violations = validator.validate_all().unwrap();

        // Should return a single PmatUnavailable info message
        assert_eq!(violations.len(), 1);
        match &violations[0] {
            PmatViolation::PmatUnavailable { severity, .. } => {
                assert_eq!(*severity, Severity::Info);
            }
            _ => panic!("Expected PmatUnavailable violation"),
        }
    }
}
