//! PMAT Integration Validator (native implementation).
//!
//! ## What
//! Validates code against PMAT (Property-Based Maintenance Analysis Tool) standards
//! natively within the Rust process, without shelling out to external binaries.
//! Checks include cyclomatic complexity, dead code detection, and technical debt scoring.
//!
//! ## Why
//! - **Performance**: Native implementation avoids process overhead.
//! - **Integration**: Direct access to AST and analysis structures.
//! - **Compliance**: Enforces code quality gates as defined in **ADR-036**.
//!
//! ## References
//! - **ADR-036**: Enforcement Layer (Policies and Guards)
//! - **ADR-033**: Native Handler Consolidation

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::constants::{DEFAULT_COMPLEXITY_THRESHOLD, DEFAULT_TDG_THRESHOLD};
use crate::pmat_native::{ComplexityAnalyzer, DeadCodeDetector, NativePmatAnalyzer, TdgScorer};
use crate::violation_trait::{Violation, ViolationCategory};
use crate::{Result, Severity, ValidationConfig};

/// PMAT violation types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PmatViolation {
    /// High cyclomatic complexity.
    ///
    /// ## Why
    /// Complex functions are harder to test, maintain, and reason about.
    /// Enforces code quality standards defined in **ADR-036 (Enforcement Layer)**.
    HighComplexity {
        /// The source file containing the complex code.
        file: PathBuf,
        /// The specific function or method identifier.
        function: String,
        /// The calculated cyclomatic complexity score.
        complexity: u32,
        /// The maximum allowed complexity before violation.
        threshold: u32,
        /// The enforcement level (Error/Warn) based on violation magnitude.
        severity: Severity,
    },
    /// Dead code detected.
    ///
    /// ## Why
    /// Unused code bloats the codebase, confuses readers, and increases maintenance burden.
    /// See **ADR-036** for policy on code hygiene.
    DeadCode {
        /// The source file containing the unused code.
        file: PathBuf,
        /// The line number where the definition starts.
        line: usize,
        /// The type of item (e.g., "function", "struct").
        item_type: String,
        /// The name of the unused item.
        name: String,
        /// The enforcement level.
        severity: Severity,
    },
    /// Low TDG score (high technical debt).
    ///
    /// ## Why
    /// Technical Debt Gradient (TDG) predicts maintenance effort.
    /// Compares code complexity against its churn/age to identify "hotspots".
    LowTdgScore {
        /// The source file being analyzed.
        file: PathBuf,
        /// The calculated Technical Debt Gradient score.
        score: u32,
        /// The minimum acceptable score (lower means higher debt).
        threshold: u32,
        /// The enforcement level.
        severity: Severity,
    },
    /// PMAT tooling unavailable (kept for compatibility).
    PmatUnavailable {
        /// Description of why the tool is unavailable.
        message: String,
        /// The enforcement level (usually Info/Warn).
        severity: Severity,
    },
    /// PMAT execution error.
    PmatError {
        /// The command or operation that failed.
        command: String,
        /// The error details.
        error: String,
        /// The enforcement level.
        severity: Severity,
    },
}

impl PmatViolation {
    /// Returns the severity level of this violation.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}

impl std::fmt::Display for PmatViolation {
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
            Self::PmatUnavailable { message, .. } => write!(f, "PMAT unavailable: {message}"),
            Self::PmatError { command, error, .. } => {
                write!(f, "PMAT error running '{command}': {error}")
            }
        }
    }
}

impl Violation for PmatViolation {
    fn id(&self) -> &str {
        match self {
            Self::HighComplexity { .. } => "PMAT001",
            Self::DeadCode { .. } => "PMAT002",
            Self::LowTdgScore { .. } => "PMAT003",
            Self::PmatUnavailable { .. } => "PMAT004",
            Self::PmatError { .. } => "PMAT005",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Pmat
    }

    fn severity(&self) -> Severity {
        match self {
            Self::HighComplexity { severity, .. }
            | Self::DeadCode { severity, .. }
            | Self::LowTdgScore { severity, .. }
            | Self::PmatUnavailable { severity, .. }
            | Self::PmatError { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::HighComplexity { file, .. }
            | Self::DeadCode { file, .. }
            | Self::LowTdgScore { file, .. } => Some(file),
            Self::PmatUnavailable { .. } | Self::PmatError { .. } => None,
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::DeadCode { line, .. } => Some(*line),
            _ => None,
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::HighComplexity {
                function,
                complexity,
                threshold,
                ..
            } => Some(format!(
                "Consider refactoring '{function}' to reduce complexity from {complexity} to below {threshold}. Split into smaller functions or simplify control flow."
            )),
            Self::DeadCode {
                item_type, name, ..
            } => Some(format!("{item_type} {name}")),
            Self::LowTdgScore {
                score, threshold, ..
            } => Some(format!(
                "Technical debt score {score} exceeds threshold {threshold}. Address code smells and reduce complexity."
            )),
            Self::PmatUnavailable { .. } => None,
            Self::PmatError { command, .. } => {
                Some(format!("Check analyzer configuration for '{command}'."))
            }
        }
    }
}

/// PMAT integration validator.
///
/// ## Why
/// Validates code quality metrics (complexity, dead code, technical debt)
/// directly within the MCB pipeline, as prescribed by **ADR-036**.
/// Uses native Rust providers instead of shelling out, for better performance and integration.
pub struct PmatValidator {
    /// Configuration for the validation context (workspace root, etc.).
    config: ValidationConfig,
    /// Configured limit for cyclomatic complexity.
    complexity_threshold: u32,
    /// Configured threshold for technical debt gradient.
    tdg_threshold: u32,
    /// Provider for complexity analysis logic (Strategy pattern).
    complexity_analyzer: Box<dyn ComplexityAnalyzer>,
    /// Provider for dead code detection logic.
    dead_code_detector: Box<dyn DeadCodeDetector>,
    /// Provider for TDG scoring logic.
    tdg_scorer: Box<dyn TdgScorer>,
}

impl PmatValidator {
    /// Creates a new PMAT validator with default configuration.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Creates a validator with custom configuration.
    pub fn with_config(config: ValidationConfig) -> Self {
        Self {
            config,
            complexity_threshold: DEFAULT_COMPLEXITY_THRESHOLD,
            tdg_threshold: DEFAULT_TDG_THRESHOLD,
            complexity_analyzer: Box::new(NativePmatAnalyzer),
            dead_code_detector: Box::new(NativePmatAnalyzer),
            tdg_scorer: Box::new(NativePmatAnalyzer),
        }
    }

    /// Sets the cyclomatic complexity threshold (builder pattern).
    #[must_use]
    pub fn with_complexity_threshold(mut self, threshold: u32) -> Self {
        self.complexity_threshold = threshold;
        self
    }

    /// Sets the Technical Debt Gradient threshold (builder pattern).
    #[must_use]
    pub fn with_tdg_threshold(mut self, threshold: u32) -> Self {
        self.tdg_threshold = threshold;
        self
    }

    /// Native analyzer path is always available.
    pub fn is_available(&self) -> bool {
        true
    }

    /// Runs all PMAT validations and returns detected violations.
    pub fn validate_all(&self) -> Result<Vec<PmatViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_complexity()?);
        violations.extend(self.validate_dead_code()?);
        violations.extend(self.validate_tdg()?);
        Ok(violations)
    }

    /// Runs cyclomatic complexity analysis using native analyzer.
    pub fn validate_complexity(&self) -> Result<Vec<PmatViolation>> {
        let findings = self
            .complexity_analyzer
            .analyze_complexity(&self.config.workspace_root, self.complexity_threshold)
            .map_err(|e| crate::ValidationError::Config(e.to_string()))?;

        Ok(findings
            .into_iter()
            .map(|f| PmatViolation::HighComplexity {
                file: f.file,
                function: f.function,
                complexity: f.complexity,
                threshold: self.complexity_threshold,
                severity: if f.complexity > self.complexity_threshold * 2 {
                    Severity::Warning
                } else {
                    Severity::Info
                },
            })
            .collect())
    }

    /// Runs dead code analysis using native analyzer.
    pub fn validate_dead_code(&self) -> Result<Vec<PmatViolation>> {
        let findings = self
            .dead_code_detector
            .detect_dead_code(&self.config.workspace_root)
            .map_err(|e| crate::ValidationError::Config(e.to_string()))?;

        Ok(findings
            .into_iter()
            .map(|f| PmatViolation::DeadCode {
                file: f.file,
                line: f.line,
                item_type: f.item_type,
                name: f.name,
                severity: Severity::Info,
            })
            .collect())
    }

    /// Runs Technical Debt Gradient analysis using native analyzer.
    pub fn validate_tdg(&self) -> Result<Vec<PmatViolation>> {
        let findings = self
            .tdg_scorer
            .score_tdg(&self.config.workspace_root, self.tdg_threshold)
            .map_err(|e| crate::ValidationError::Config(e.to_string()))?;

        Ok(findings
            .into_iter()
            .map(|f| PmatViolation::LowTdgScore {
                file: f.file,
                score: f.score,
                threshold: self.tdg_threshold,
                severity: if f.score > self.tdg_threshold + 25 {
                    Severity::Warning
                } else {
                    Severity::Info
                },
            })
            .collect())
    }
}

impl crate::validator_trait::Validator for PmatValidator {
    fn name(&self) -> &'static str {
        "pmat"
    }

    fn description(&self) -> &'static str {
        "Native PMAT-style analysis for cyclomatic complexity, dead code detection, and TDG scoring"
    }

    fn validate(&self, _config: &ValidationConfig) -> anyhow::Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate_all()?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }

    fn enabled_by_default(&self) -> bool {
        true
    }
}
